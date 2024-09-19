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
    GithubEvent(payload): GithubEvent<String>,
) {
    let Some(Ok(event_type)) = headers.get("X-GitHub-Event").map(|v| v.to_str()) else {
        error!("missing or invalid X-GitHub-Event header");
        return;
    };

    let event = match WebhookEvent::try_from_header_and_body(event_type, &payload) {
        Ok(event) => event,
        Err(e) => {
            error!(%e, "failed to parse event");
            return;
        }
    };

    let hook = match hook_target(&event) {
        HookTarget::Normal => {
            info!(
                hook = &app_state.discord_hooks.normal,
                "sending normal hook"
            );
            &app_state.discord_hooks.normal
        }
        HookTarget::Bot => {
            info!(hook = &app_state.discord_hooks.bot, "sending bot hook");
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

    let name = repo.full_name.unwrap_or(repo.name);

    embed.title(&format!(
        "[{}] Repository {}",
        name,
        match specifics.action {
            RepositoryWebhookEventAction::Archived => "archived".to_string(),
            RepositoryWebhookEventAction::Created => "created".to_string(),
            RepositoryWebhookEventAction::Deleted => "deleted".to_string(),
            RepositoryWebhookEventAction::Renamed => {
                format!(
                    "renamed from {} to {}",
                    name,
                    specifics
                        .changes
                        .as_ref()
                        .expect("repository renamed event should always have changes")
                        .name
                        .as_ref()
                        .expect("repository renamed event changes should always have a name")
                        .from
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
                        .login,
                    event
                        .sender
                        .expect("repository transferred event should always have a sender")
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

    embed.color(match specifics.action {
        IssuesWebhookEventAction::Closed | IssuesWebhookEventAction::Locked => COLORS.red,
        _ => ISSUE_COLOR,
    });

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

    embed.color(match specifics.action {
        PullRequestWebhookEventAction::Closed | PullRequestWebhookEventAction::Locked => COLORS.red,
        PullRequestWebhookEventAction::Opened | PullRequestWebhookEventAction::ReadyForReview => {
            COLORS.green
        }
        _ => PULL_REQUEST_COLOR,
    });

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

    embed.color(ISSUE_COLOR);

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

    embed.color(REPO_COLOR);

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

    embed.color(COLORS.base);

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

    use crate::make_embed;

    #[test]
    fn test_bot_pull_request_opened() {
        let payload = include_str!("../fixtures/bot_pull_request_opened.json");
        let event = WebhookEvent::try_from_header_and_body("pull_request", payload)
            .expect("event fixture is valid");
        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");
        assert_eq!(
            embed["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin-rfc/cli-old] Pull request opened: #1 chore: Configure Renovate"
        );
    }

    #[test]
    fn test_limit_description_on_pull_request() {
        let payload = include_str!("../fixtures/bot_pull_request_opened.json");
        let event = WebhookEvent::try_from_header_and_body("pull_request", payload)
            .expect("event fixture is valid");
        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");
        assert_eq!(
            embed["embeds"][0]["description"]
                .as_str()
                .unwrap()
                .split_once('!')
                .unwrap()
                .0,
            "Welcome to [Renovate](https://redirect.github.com/renovatebot/renovate)"
        );
        assert_eq!(
            embed["embeds"][0]["description"].as_str().unwrap().len(),
            640
        );
    }

    #[test]
    fn test_ignore_pr_events() {
        let payload = include_str!("../fixtures/pull_request_synchronize.json");
        let event = WebhookEvent::try_from_header_and_body("pull_request", payload)
            .expect("event fixture is valid");
        let embed = make_embed(event).expect("make_embed should succeed");
        assert!(embed.is_none());
    }

    #[test]
    fn test_issue_opened() {
        let payload = include_str!("../fixtures/issue_opened.json");
        let event = WebhookEvent::try_from_header_and_body("issues", payload)
            .expect("event fixture is valid");
        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");
        assert_eq!(
            embed["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin/userstyles] Issue opened: #1318 LinkedIn: Profile picture edition icons and text is u..."
        );
    }

    #[test]
    fn test_ignore_issue_events() {
        let payload = include_str!("../fixtures/issue_unassigned.json");
        let event = WebhookEvent::try_from_header_and_body("issues", payload)
            .expect("event fixture is valid");
        let embed = make_embed(event).expect("make_embed should succeed");
        assert!(embed.is_none());
    }

    mod issue_comment {
        use octocrab::models::webhook_events::WebhookEvent;

        use crate::make_embed;

        #[test]
        fn created() {
            let payload = include_str!("../fixtures/issue_comment/created.json");
            let event = WebhookEvent::try_from_header_and_body("issue_comment", payload)
                .expect("event fixture is valid");
            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin/java] New comment on issue #20: Reconsider OSSRH Authentication"
            );
        }

        #[test]
        fn created_on_pull_request() {
            let payload = include_str!("../fixtures/issue_comment/created_on_pull_request.json");
            let event = WebhookEvent::try_from_header_and_body("issue_comment", payload)
                .expect("event fixture is valid");
            let embed = make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin/userstyles] New comment on pull request #1323: feat(fontawesome): init"
            );
        }

        #[test]
        fn deleted() {
            let payload = include_str!("../fixtures/issue_comment/deleted.json");
            let event = WebhookEvent::try_from_header_and_body("issue_comment", payload)
                .expect("event fixture is valid");
            let embed = make_embed(event).expect("make_embed should succeed");
            assert!(embed.is_none());
        }
    }

    mod pull_request_review {
        use octocrab::models::webhook_events::WebhookEvent;

        #[test]
        fn approved() {
            let payload = include_str!("../fixtures/pull_request_review/approved.json");
            let event = WebhookEvent::try_from_header_and_body("pull_request_review", payload)
                .expect("event fixture is valid");
            let embed = crate::make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/polybar] Pull request approved: #3 chore: Configure Renovate"
            );
        }

        #[test]
        fn changes_requested() {
            let payload = include_str!("../fixtures/pull_request_review/changes_requested.json");
            let event = WebhookEvent::try_from_header_and_body("pull_request_review", payload)
                .expect("event fixture is valid");
            let embed = crate::make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
            embed["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin-rfc/polybar] Pull request changes requested: #3 chore: Configure Renovate"
        );
            assert_eq!(embed["embeds"][0]["description"].as_str().unwrap(), "test");
        }

        #[test]
        fn commented() {
            let payload = include_str!("../fixtures/pull_request_review/commented.json");
            let event = WebhookEvent::try_from_header_and_body("pull_request_review", payload)
                .expect("event fixture is valid");
            let embed = crate::make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/polybar] Pull request reviewed: #3 chore: Configure Renovate"
            );
            assert_eq!(
                embed["embeds"][0]["description"].as_str().unwrap(),
                "normal"
            );
        }
    }

    mod membership {
        use octocrab::models::webhook_events::WebhookEvent;

        #[test]
        fn added() {
            let payload = include_str!("../fixtures/membership/added.json");
            let event = WebhookEvent::try_from_header_and_body("membership", payload)
                .expect("event fixture is valid");
            let embed = crate::make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc] backwardspy added to staff team"
            );
        }

        #[test]
        fn removed() {
            let payload = include_str!("../fixtures/membership/removed.json");
            let event = WebhookEvent::try_from_header_and_body("membership", payload)
                .expect("event fixture is valid");
            let embed = crate::make_embed(event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");
            assert_eq!(
                embed["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc] backwardspy removed from staff team"
            );
        }
    }
}
