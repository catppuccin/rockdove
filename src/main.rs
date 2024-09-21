use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    http::HeaderMap,
    routing::post,
    Router,
};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use octocrab::models::{
    pulls::ReviewState,
    webhook_events::{
        payload::{
            IssueCommentWebhookEventAction, IssueCommentWebhookEventPayload,
            IssuesWebhookEventAction, IssuesWebhookEventPayload, MembershipWebhookEventAction,
            MembershipWebhookEventPayload, PullRequestReviewWebhookEventAction,
            PullRequestReviewWebhookEventPayload, PullRequestWebhookEventAction,
            PullRequestWebhookEventPayload, ReleaseWebhookEventAction, ReleaseWebhookEventPayload,
            RepositoryWebhookEventAction, RepositoryWebhookEventPayload,
        },
        WebhookEvent, WebhookEventPayload,
    },
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, warn, Level};

mod embed_builder;
use embed_builder::EmbedBuilder;

const COLORS: catppuccin::FlavorColors = catppuccin::PALETTE.mocha.colors;
const ISSUE_COLOR: catppuccin::Color = COLORS.green;
const PULL_REQUEST_COLOR: catppuccin::Color = COLORS.blue;
const REPO_COLOR: catppuccin::Color = COLORS.yellow;
const RELEASE_COLOR: catppuccin::Color = COLORS.mauve;
const MEMBERSHIP_COLOR: catppuccin::Color = COLORS.base;

#[derive(serde::Deserialize)]
struct Config {
    github_webhook_secret: String,
    discord_webhook: String,
    discord_bot_webhook: String,
    #[serde(default = "default_port")]
    port: u16,
}

const fn default_port() -> u16 {
    3000
}

#[derive(Clone)]
struct DiscordHooks {
    normal: String,
    bot: String,
}

#[derive(Clone)]
struct AppState {
    discord_hooks: DiscordHooks,
    github_token: GithubToken,
}

impl FromRef<AppState> for GithubToken {
    fn from_ref(state: &AppState) -> Self {
        state.github_token.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .init();

    let config: Config = envy::from_env()?;

    let app = Router::new()
        .route("/webhook", post(webhook))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(AppState {
            discord_hooks: DiscordHooks {
                normal: config.discord_webhook,
                bot: config.discord_bot_webhook,
            },
            github_token: GithubToken(Arc::new(config.github_webhook_secret)),
        });

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    let addr = listener.local_addr()?;
    info!(?addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}

enum HookTarget {
    Normal,
    Bot,
    None,
}

async fn webhook(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    GithubEvent(payload): GithubEvent<serde_json::Value>,
) {
    let Some(Ok(event_type)) = headers.get("X-GitHub-Event").map(|v| v.to_str()) else {
        error!("missing or invalid X-GitHub-Event header");
        return;
    };

    info!(?event_type, "received event");

    let event = match WebhookEvent::try_from_header_and_body(event_type, &payload.to_string()) {
        Ok(event) => event,
        Err(e) => {
            error!(%e, "failed to parse event");
            return;
        }
    };

    let hook = match hook_target(&event) {
        HookTarget::Normal => {
            info!("hook target is normal");
            &app_state.discord_hooks.normal
        }
        HookTarget::Bot => {
            info!("hook target is bot");
            &app_state.discord_hooks.bot
        }
        HookTarget::None => {
            info!("no target - ignoring event");
            return;
        }
    };

    match make_embed(event) {
        Ok(Some(msg)) => send_hook(&msg, hook).await,
        Ok(None) => info!("no embed created - ignoring event"),
        Err(e) => error!(%e, "failed to make discord message"),
    }
}

#[allow(clippy::too_many_lines)]
fn make_embed(event: WebhookEvent) -> anyhow::Result<Option<serde_json::Value>> {
    let sender = event
        .sender
        .clone()
        .expect("event should always have a sender");

    let Some(mut embed) = (match event.specific.clone() {
        WebhookEventPayload::Repository(specifics) => make_repository_embed(event, &specifics),
        WebhookEventPayload::Issues(specifics) => make_issue_embed(event, &specifics),
        WebhookEventPayload::PullRequest(specifics) => make_pull_request_embed(event, &specifics),
        WebhookEventPayload::IssueComment(specifics) => make_issue_comment_embed(event, &specifics),
        WebhookEventPayload::PullRequestReview(specifics) => {
            make_pull_request_review_embed(event, &specifics)
        }
        WebhookEventPayload::Release(specifics) => make_release_embed(event, &specifics),
        WebhookEventPayload::Membership(specifics) => make_membership_embed(event, &specifics),
        _ => {
            info!(?event.kind, "ignoring event");
            return Ok(None);
        }
    }) else {
        return Ok(None);
    };

    embed.author(sender);
    Ok(Some(embed.try_build()?))
}

fn make_repository_embed(
    event: WebhookEvent,
    specifics: &RepositoryWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("repository events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let full_name = repo.full_name.as_ref().unwrap_or(&repo.name);

    embed.title(&format!(
        "[{}] Repository {}",
        full_name,
        match specifics.action {
            RepositoryWebhookEventAction::Archived => "archived".to_string(),
            RepositoryWebhookEventAction::Created => "created".to_string(),
            RepositoryWebhookEventAction::Deleted => "deleted".to_string(),
            RepositoryWebhookEventAction::Renamed => {
                format!(
                    "renamed from {} to {}",
                    specifics
                        .changes
                        .as_ref()
                        .expect("repository renamed event should always have changes")
                        .repository
                        .as_ref()
                        .expect("repository renamed event changes should always have a repository")
                        .name
                        .as_ref()
                        .expect("repository renamed event changes should always have a name")
                        .from,
                    repo.name,
                )
            }
            RepositoryWebhookEventAction::Transferred => {
                format!(
                    "transferred from {} to {}",
                    specifics
                        .changes
                        .as_ref()
                        .expect("repository transferred event should always have changes")
                        .owner
                        .as_ref()
                        .expect("repository transferred event changes should always have an owner")
                        .from
                        .user
                        .login,
                    repo.owner
                        .expect("repository should always have an owner")
                        .login
                )
            }
            RepositoryWebhookEventAction::Unarchived => "unarchived".to_string(),
            _ => {
                return None;
            }
        }
    ));

    embed.url(
        repo.html_url
            .expect("repository should always have an html url")
            .as_str(),
    );

    embed.color(match specifics.action {
        RepositoryWebhookEventAction::Deleted => COLORS.red,
        RepositoryWebhookEventAction::Transferred => COLORS.pink,
        _ => REPO_COLOR,
    });

    Some(embed)
}

fn make_issue_embed(
    event: WebhookEvent,
    specifics: &IssuesWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("issue events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Issue {}: #{} {}",
        repo_name,
        match specifics.action {
            IssuesWebhookEventAction::Assigned => {
                let assignee = specifics
                    .issue
                    .assignee
                    .as_ref()
                    .expect("issue assigned events should always have an assignee");
                format!("assigned to {}", assignee.login)
            }
            IssuesWebhookEventAction::Closed => "closed".to_string(),
            IssuesWebhookEventAction::Locked => "locked".to_string(),
            IssuesWebhookEventAction::Opened => "opened".to_string(),
            IssuesWebhookEventAction::Pinned => "pinned".to_string(),
            IssuesWebhookEventAction::Reopened => "reopened".to_string(),
            // TODO: this would be nice
            // IssuesWebhookEventAction::Transferred => {
            //     todo!()
            // }
            _ => {
                return None;
            }
        },
        specifics.issue.number,
        specifics.issue.title,
    ));

    embed.url(specifics.issue.html_url.as_str());

    if matches!(specifics.action, IssuesWebhookEventAction::Opened) {
        if let Some(ref body) = specifics.issue.body {
            embed.description(body);
        }
    }

    embed.color(ISSUE_COLOR);

    Some(embed)
}

fn make_pull_request_embed(
    event: WebhookEvent,
    specifics: &PullRequestWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("pull request events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Pull request {}: #{} {}",
        repo_name,
        match specifics.action {
            PullRequestWebhookEventAction::Assigned => {
                let assignee = specifics
                    .assignee
                    .as_ref()
                    .expect("pull request assigned events should always have an assignee");
                format!("assigned to {}", assignee.login)
            }
            PullRequestWebhookEventAction::Closed => {
                if specifics.pull_request.merged_at.is_some() {
                    "merged".to_string()
                } else {
                    "closed".to_string()
                }
            }
            PullRequestWebhookEventAction::Locked => "locked".to_string(),
            PullRequestWebhookEventAction::Opened => "opened".to_string(),
            PullRequestWebhookEventAction::ReadyForReview => "ready for review".to_string(),
            PullRequestWebhookEventAction::Reopened => "reopened".to_string(),
            PullRequestWebhookEventAction::ReviewRequested => {
                let reviewer = specifics
                    .requested_reviewer
                    .as_ref()
                    .expect("pull request review requested events should always have a reviewer");
                format!("review requested from {}", reviewer.login)
            }
            _ => {
                return None;
            }
        },
        specifics.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .expect("pull request should always have a title")
    ));

    embed.url(
        specifics
            .pull_request
            .html_url
            .as_ref()
            .expect("pull request should always have an html url")
            .as_str(),
    );

    if matches!(specifics.action, PullRequestWebhookEventAction::Opened) {
        if let Some(ref body) = specifics.pull_request.body {
            embed.description(body);
        }
    }

    embed.color(PULL_REQUEST_COLOR);

    Some(embed)
}

fn make_issue_comment_embed(
    event: WebhookEvent,
    specifics: &IssueCommentWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(specifics.action, IssueCommentWebhookEventAction::Created) {
        return None;
    }

    let repo = event
        .repository
        .expect("issue comment events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);
    let target = if specifics.issue.pull_request.is_some() {
        "pull request"
    } else {
        "issue"
    };

    embed.title(&format!(
        "[{}] New comment on {} #{}: {}",
        repo_name, target, specifics.issue.number, specifics.issue.title,
    ));

    embed.url(specifics.comment.html_url.as_str());

    if let Some(ref body) = specifics.comment.body {
        embed.description(body);
    }

    embed.color(if specifics.issue.pull_request.is_some() {
        PULL_REQUEST_COLOR
    } else {
        ISSUE_COLOR
    });

    Some(embed)
}

fn make_pull_request_review_embed(
    event: WebhookEvent,
    specifics: &PullRequestReviewWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(
        specifics.action,
        PullRequestReviewWebhookEventAction::Submitted
    ) {
        return None;
    }

    let repo = event
        .repository
        .expect("pull request review events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Pull request {}: #{} {}",
        repo_name,
        match specifics
            .review
            .state
            .expect("pull request review should always have a state")
        {
            ReviewState::Approved => "approved",
            ReviewState::ChangesRequested => "changes requested",
            ReviewState::Commented => "reviewed",
            _ => return None,
        },
        specifics.pull_request.number,
        specifics
            .pull_request
            .title
            .as_ref()
            .expect("pull request should always have a title"),
    ));

    embed.url(specifics.review.html_url.as_str());

    if let Some(ref body) = specifics.review.body {
        embed.description(body);
    }

    embed.color(PULL_REQUEST_COLOR);

    Some(embed)
}

fn make_release_embed(
    event: WebhookEvent,
    specifics: &ReleaseWebhookEventPayload,
) -> Option<EmbedBuilder> {
    if !matches!(specifics.action, ReleaseWebhookEventAction::Released) {
        return None;
    }

    let repo = event
        .repository
        .expect("release events should always have a repository");

    let mut embed = EmbedBuilder::default();

    let repo_name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] New release published: {}",
        repo_name,
        specifics
            .release
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("*no name*"),
    ));

    embed.url(
        specifics
            .release
            .get("html_url")
            .and_then(|v| v.as_str())
            .expect("release should always have an html url"),
    );

    if let Some(body) = specifics.release.get("body").and_then(|v| v.as_str()) {
        embed.description(body);
    }

    embed.color(RELEASE_COLOR);

    Some(embed)
}

fn make_membership_embed(
    event: WebhookEvent,
    specifics: &MembershipWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let Some(team_name) = specifics.team.get("name").and_then(|v| v.as_str()) else {
        warn!(?specifics.team, "missing team name");
        return None;
    };

    let Some(member_login) = specifics.member.get("login").and_then(|v| v.as_str()) else {
        warn!(?specifics.member, "missing member login");
        return None;
    };

    let mut embed = EmbedBuilder::default();

    embed.title(&format!(
        "[{}] {} {} {} team",
        event.organization?.login,
        member_login,
        match specifics.action {
            MembershipWebhookEventAction::Added => "added to",
            MembershipWebhookEventAction::Removed => "removed from",
            _ => {
                return None;
            }
        },
        team_name
    ));

    embed.url(
        specifics
            .team
            .get("html_url")
            .and_then(|v| v.as_str())
            .expect("team should always have an html url"),
    );

    embed.color(MEMBERSHIP_COLOR);

    Some(embed)
}

async fn send_hook(e: &serde_json::Value, hook: &str) {
    match reqwest::Client::new().post(hook).json(e).send().await {
        Err(e) => error!(%e, "failed to send hook"),
        Ok(r) => {
            if let Err(e) = r.error_for_status() {
                error!(%e, "hook failed");
            } else {
                info!("hook sent");
            }
        }
    }
}

fn hook_target(event: &WebhookEvent) -> HookTarget {
    if let Some(sender) = &event.sender {
        if sender.r#type == "Bot" {
            return HookTarget::Bot;
        }
    }

    if let Some(repository) = &event.repository {
        if repository.private.unwrap_or(false) {
            info!("ignoring private repository event");
            return HookTarget::None;
        }
    }

    HookTarget::Normal
}

#[cfg(test)]
mod tests {
    use octocrab::models::webhook_events::WebhookEvent;
    use serde_json::json;

    struct TestConfig {
        event: WebhookEvent,
        settings: insta::Settings,
    }

    impl TestConfig {
        fn new(event_type: &str, payload: &str) -> Self {
            let event = WebhookEvent::try_from_header_and_body(event_type, payload)
                .expect("event fixture is valid");
            let mut settings = insta::Settings::new();
            settings.set_omit_expression(true);
            settings.set_snapshot_path(format!("../snapshots/{event_type}"));
            settings.set_prepend_module_to_snapshot(false);
            Self { event, settings }
        }
    }

    fn embed_context(embed: &serde_json::Value) -> serde_json::Value {
        json!({
            "author_name_length": &embed["embeds"][0]["author"]["name"].as_str().unwrap().len(),
            "title_length": &embed["embeds"][0]["title"].as_str().unwrap().len(),
            "description_length": &embed["embeds"][0]["description"].as_str().unwrap_or("").len(),
            "colour_hex": format!("#{:X}", embed["embeds"][0]["color"].as_u64().unwrap()),
        })
    }

    mod pull_request {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn opened() {
            let payload = include_str!("../fixtures/pull_request/opened.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn opened_by_bot() {
            let payload = include_str!("../fixtures/pull_request/opened_by_bot.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn closed() {
            let payload = include_str!("../fixtures/pull_request/closed.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn reopened() {
            let payload = include_str!("../fixtures/pull_request/reopened.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }

    mod issues {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn opened() {
            let payload = include_str!("../fixtures/issues/opened.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("issues", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn closed() {
            let payload = include_str!("../fixtures/issues/closed.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("issues", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn reopened() {
            let payload = include_str!("../fixtures/issues/reopened.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("issues", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }

    mod repository {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn created() {
            let payload = include_str!("../fixtures/repository/created.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("repository", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn renamed() {
            let payload = include_str!("../fixtures/repository/renamed.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("repository", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn transferred() {
            let payload = include_str!("../fixtures/repository/transferred.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("repository", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }

    mod pull_request_review {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn approved() {
            let payload = include_str!("../fixtures/pull_request_review/approved.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request_review", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn changes_requested() {
            let payload = include_str!("../fixtures/pull_request_review/changes_requested.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request_review", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn commented() {
            let payload = include_str!("../fixtures/pull_request_review/commented.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("pull_request_review", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }

    mod issue_comment {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn created() {
            let payload = include_str!("../fixtures/issue_comment/created.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("issue_comment", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn created_on_pull_request() {
            let payload = include_str!("../fixtures/issue_comment/created_on_pull_request.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("issue_comment", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }

    mod membership {
        use crate::{
            make_embed,
            tests::{embed_context, TestConfig},
        };

        #[test]
        fn added() {
            let payload = include_str!("../fixtures/membership/added.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("membership", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }

        #[test]
        fn removed() {
            let payload = include_str!("../fixtures/membership/removed.json");
            let TestConfig {
                event,
                mut settings,
            } = super::TestConfig::new("membership", payload);

            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        }
    }
}
