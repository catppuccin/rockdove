use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    sync::Arc,
};

use axum::{
    extract::{FromRef, State},
    http::HeaderMap,
    routing::post,
    Router,
};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde_json::json;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};

const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 640;
const MAX_AUTHOR_NAME_LENGTH: usize = 256;

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

// https://docs.github.com/en/webhooks/webhook-events-and-payloads
#[derive(serde::Deserialize)]
struct Event {
    action: String,
    sender: User,
    member: Option<User>,
    repository: Option<Repository>,
    issue: Option<Issue>,
    comment: Option<Comment>,
    pull_request: Option<PullRequest>,
    review: Option<PullRequestReview>,
    team: Option<Team>,
    organization: Option<Organization>,
    release: Option<Release>,
    changes: Option<Changes>,
}

#[derive(serde::Deserialize)]
struct Repository {
    full_name: String,
    name: String,
    html_url: String,
    private: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct User {
    login: String,
    avatar_url: String,
    html_url: String,
    #[serde(rename = "type")]
    sender_type: String,
}

#[derive(Clone, serde::Deserialize)]
struct Comment {
    body: String,
    html_url: String,
}

#[derive(serde::Deserialize)]
struct Issue {
    title: String,
    number: u64,
    html_url: String,
    body: Option<String>,
    pull_request: Option<IssueCommentPullRequest>,
}

/// "This event occurs when there is activity relating to a comment on an issue
/// or pull request."
///
/// <https://docs.github.com/en/webhooks/webhook-events-and-payloads#issue_comment>
#[derive(serde::Deserialize)]
struct IssueCommentPullRequest {}

#[derive(serde::Deserialize)]
struct PullRequest {
    title: String,
    number: u64,
    html_url: String,
    body: Option<String>,
    merged_at: Option<String>,
}

#[derive(serde::Deserialize)]
struct PullRequestReview {
    html_url: String,
    body: Option<String>,
    state: String,
}

#[derive(serde::Deserialize)]
struct Release {
    html_url: String,
    name: String,
}

#[derive(serde::Deserialize)]
struct Team {
    name: String,
    html_url: String,
}

#[derive(serde::Deserialize)]
struct Organization {
    login: String,
}

#[derive(serde::Deserialize)]
struct Changes {
    owner: Option<ChangesOwner>,
    repository: Option<ChangesRepository>,
}

#[derive(serde::Deserialize)]
struct ChangesOwner {
    from: ChangesOwnerFrom,
}

#[derive(serde::Deserialize)]
struct ChangesOwnerFrom {
    user: User,
}

#[derive(serde::Deserialize)]
struct ChangesRepository {
    name: ChangesRepositoryName,
}

#[derive(serde::Deserialize)]
struct ChangesRepositoryName {
    from: String,
}

async fn webhook(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    GithubEvent(e): GithubEvent<Event>,
) {
    if e.action == "edited" {
        info!("ignoring edited event");
        return;
    }

    let Some(Ok(event_type)) = headers.get("X-GitHub-Event").map(|v| v.to_str()) else {
        error!("missing or invalid X-GitHub-Event header");
        return;
    };

    match hook_target(&e) {
        HookTarget::Normal => {
            info!(
                hook = &app_state.discord_hooks.normal,
                "sending normal hook"
            );
            match make_discord_message(event_type, &e) {
                Ok(Some(msg)) => send_hook(&msg, &app_state.discord_hooks.normal).await,
                Ok(None) => info!("no embed created - ignoring event"),
                Err(e) => error!(%e, "failed to make discord message"),
            }
        }
        HookTarget::Bot => {
            info!(hook = &app_state.discord_hooks.bot, "sending bot hook");
            match make_discord_message(event_type, &e) {
                Ok(Some(msg)) => send_hook(&msg, &app_state.discord_hooks.bot).await,
                Ok(None) => info!("no embed created - ignoring event"),
                Err(e) => error!(%e, "failed to make discord message"),
            }
        }
        HookTarget::None => info!("no target - ignoring event"),
    }
}

#[derive(Default, Debug)]
struct EmbedBuilder {
    title: Option<String>,
    url: Option<String>,
    author: Option<User>,
    description: Option<String>,
    color: Option<u32>,
}

impl EmbedBuilder {
    fn title(&mut self, title: &str) -> &Self {
        self.title = Some(limit_text_length(title, MAX_TITLE_LENGTH));
        self
    }

    fn url(&mut self, url: &str) -> &Self {
        self.url = Some(url.to_string());
        self
    }

    fn author(&mut self, author: User) -> &Self {
        self.author = Some(author);
        self
    }

    fn description(&mut self, description: &str) -> &Self {
        self.description = Some(limit_text_length(description, MAX_DESCRIPTION_LENGTH));
        self
    }

    fn color(&mut self, color: catppuccin::Color) -> &Self {
        let rgb = color.rgb;
        self.color = Some(u32::from(rgb.r) << 16 | u32::from(rgb.g) << 8 | u32::from(rgb.b));
        self
    }

    fn try_build(self) -> anyhow::Result<serde_json::Value> {
        Ok(json!({
            "embeds": [{
                "title": self.title.ok_or_else(|| anyhow::anyhow!("missing title"))?,
                "url": self.url.ok_or_else(|| anyhow::anyhow!("missing url"))?,
                "description": self.description,
                "color": self.color,
                "author": embed_author(&self.author.ok_or_else(|| anyhow::anyhow!("missing author"))?),
            }],
        }))
    }
}

fn embed_author(user: &User) -> serde_json::Value {
    json!({
        "name": limit_text_length(&user.login, MAX_AUTHOR_NAME_LENGTH),
        "url": user.html_url,
        "icon_url": user.avatar_url,
    })
}

fn limit_text_length(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        format!("{}...", &text[..max_length - 3])
    } else {
        text.to_string()
    }
}

#[allow(clippy::too_many_lines)]
fn make_discord_message(event_type: &str, e: &Event) -> anyhow::Result<Option<serde_json::Value>> {
    let mut embed = EmbedBuilder::default();

    embed.color(pick_color(event_type));
    embed.author(e.sender.clone());

    let display_action = e.action.replace('_', " ");

    if let Some(repository) = &e.repository {
        if let Some(comment) = &e.comment {
            if e.action != "created" {
                return Ok(None);
            }

            if let Some(issue) = &e.issue {
                let action = if issue.pull_request.is_some() {
                    "pull request"
                } else {
                    "issue"
                };

                embed.title(&format!(
                    "[{}] New comment on {} #{}: {}",
                    repository.full_name, action, issue.number, issue.title
                ));
                embed.url(&comment.html_url);
                embed.description(&comment.body);
            } else {
                return Ok(None);
            }
        } else if let Some(issue) = &e.issue {
            if e.action != "opened"
                && e.action != "closed"
                && e.action != "reopened"
                && e.action != "transferred"
            {
                return Ok(None);
            }

            embed.title(&format!(
                "[{}] Issue {}: #{} {}",
                repository.full_name, display_action, issue.number, issue.title
            ));

            if e.action == "opened" {
                if let Some(body) = &issue.body {
                    embed.description(body);
                }
            }

            embed.url(&issue.html_url);
        } else if let Some(pull_request) = &e.pull_request {
            if let Some(pull_request_review) = &e.review {
                if e.action != "submitted" {
                    return Ok(None);
                }

                let action = match pull_request_review.state.as_str() {
                    "approved" => "approved",
                    "changes_requested" => "changes requested",
                    "commented" => "reviewed",
                    _ => pull_request_review.state.as_str(),
                };

                embed.title(&format!(
                    "[{}] Pull request {}: #{} {}",
                    repository.full_name, action, pull_request.number, pull_request.title
                ));

                if let Some(body) = &pull_request_review.body {
                    embed.description(body);
                }

                embed.url(&pull_request_review.html_url);
            } else {
                if e.action != "opened" && e.action != "closed" && e.action != "reopened" {
                    return Ok(None);
                }

                let action = if e.action == "closed" && pull_request.merged_at.is_some() {
                    "merged"
                } else {
                    &display_action
                };

                embed.title(&format!(
                    "[{}] Pull request {}: #{} {}",
                    repository.full_name, action, pull_request.number, pull_request.title
                ));

                if e.action == "opened" {
                    if let Some(body) = &pull_request.body {
                        embed.description(body);
                    }
                }

                embed.url(&pull_request.html_url);
            }
        } else if let Some(release) = &e.release {
            if e.action != "released" {
                return Ok(None);
            }

            embed.title(&format!(
                "[{}] New release published: {}",
                repository.full_name, release.name
            ));
            embed.url(&release.html_url);
        } else if let Some(changes) = &e.changes {
            if let Some(ChangesOwner {
                from: ChangesOwnerFrom { user },
            }) = &changes.owner
            {
                embed.title(&format!(
                    "[{}] Repository transferred from {}/{}",
                    repository.full_name, user.login, repository.name
                ));
                embed.url(&repository.html_url);
            } else if let Some(ChangesRepository {
                name: ChangesRepositoryName { from },
            }) = &changes.repository
            {
                embed.title(&format!(
                    "[{}] Repository renamed from {}",
                    repository.full_name, from
                ));
                embed.url(&repository.html_url);
            } else {
                return Ok(None);
            }
        } else if matches!(e.action.as_str(), "archived" | "unarchived") {
            embed.title(&format!(
                "[{}] Repository {}",
                repository.full_name, e.action
            ));
            embed.url(&repository.html_url);
        } else {
            return Ok(None);
        }
    } else if let Some(team) = &e.team {
        let org = e
            .organization
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing organization"))?;
        let member = e
            .member
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing member"))?;

        embed.title(&format!(
            "[{}/{}] Member {} {}",
            org.login, team.name, member.login, e.action
        ));
        embed.url(&team.html_url);
    }

    Ok(Some(embed.try_build()?))
}

fn pick_color(event_type: &str) -> catppuccin::Color {
    let options = catppuccin::PALETTE
        .mocha
        .colors
        .iter()
        .filter(|c| c.accent)
        .collect::<Vec<_>>();

    let mut hasher = DefaultHasher::new();
    event_type.hash(&mut hasher);
    let hash = hasher.finish();

    #[allow(clippy::cast_possible_truncation)]
    let choice = hash as usize % options.len();

    *options[choice]
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

fn hook_target(e: &Event) -> HookTarget {
    if e.sender.sender_type == "Bot" {
        return HookTarget::Bot;
    }

    if let Some(repository) = &e.repository {
        if repository.private {
            info!("ignoring private repository event");
            return HookTarget::None;
        }
    }

    HookTarget::Normal
}

#[cfg(test)]
mod tests {
    use crate::{make_discord_message, Event};

    #[test]
    fn test_bot_pull_request_opened() {
        let payload = include_str!("../fixtures/bot_pull_request_opened.json");
        let e: Event = serde_json::from_str(payload).unwrap();
        let msg = make_discord_message("pull_request", &e).unwrap().unwrap();
        assert_eq!(
            msg["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin-rfc/cli-old] Pull request opened: #1 chore: Configure Renovate"
        );
    }

    #[test]
    fn test_limit_description_on_pull_request() {
        let payload = include_str!("../fixtures/bot_pull_request_opened.json");
        let e: Event = serde_json::from_str(payload).unwrap();
        let msg = make_discord_message("pull_request", &e).unwrap().unwrap();
        assert_eq!(
            msg["embeds"][0]["description"]
                .as_str()
                .unwrap()
                .split_once('!')
                .unwrap()
                .0,
            "Welcome to [Renovate](https://redirect.github.com/renovatebot/renovate)"
        );
        assert_eq!(msg["embeds"][0]["description"].as_str().unwrap().len(), 640);
    }

    #[test]
    fn test_ignore_pr_events() {
        let payload = include_str!("../fixtures/pull_request_synchronize.json");
        let e: Event = serde_json::from_str(payload).unwrap();
        let msg = make_discord_message("pull_request", &e).unwrap();
        assert!(msg.is_none());
    }

    #[test]
    fn test_issue_opened() {
        let payload = include_str!("../fixtures/issue_opened.json");
        let e: Event = serde_json::from_str(payload).unwrap();
        let msg = make_discord_message("issues", &e).unwrap().unwrap();
        assert_eq!(
            msg["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin/userstyles] Issue opened: #1318 LinkedIn: Profile picture edition icons and text is u..."
        );
    }

    #[test]
    fn test_ignore_issue_events() {
        let payload = include_str!("../fixtures/issue_unassigned.json");
        let e: Event = serde_json::from_str(payload).unwrap();
        let msg = make_discord_message("issues", &e).unwrap();
        assert!(msg.is_none());
    }

    mod issue_comment {
        use crate::{make_discord_message, Event};

        #[test]
        fn created() {
            let payload = include_str!("../fixtures/issue_comment/created.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("issue_comment", &e).unwrap().unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin/java] New comment on issue #20: Reconsider OSSRH Authentication"
            );
        }

        #[test]
        fn created_on_pull_request() {
            let payload = include_str!("../fixtures/issue_comment/created_on_pull_request.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("issue_comment", &e).unwrap().unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin/userstyles] New comment on pull request #1323: feat(fontawesome): init"
            );
        }

        #[test]
        fn deleted() {
            let payload = include_str!("../fixtures/issue_comment/deleted.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("issue_comment", &e).unwrap();
            assert!(msg.is_none());
        }
    }

    mod pull_request_review {
        use crate::{make_discord_message, Event};

        #[test]
        fn approved() {
            let payload = include_str!("../fixtures/pull_request_review/approved.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("pull_request_review", &e)
                .unwrap()
                .unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/polybar] Pull request approved: #3 chore: Configure Renovate"
            );
        }

        #[test]
        fn changes_requested() {
            let payload = include_str!("../fixtures/pull_request_review/changes_requested.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("pull_request_review", &e)
                .unwrap()
                .unwrap();
            assert_eq!(
            msg["embeds"][0]["title"].as_str().unwrap(),
            "[catppuccin-rfc/polybar] Pull request changes requested: #3 chore: Configure Renovate"
        );
            assert_eq!(msg["embeds"][0]["description"].as_str().unwrap(), "test");
        }

        #[test]
        fn commented() {
            let payload = include_str!("../fixtures/pull_request_review/commented.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("pull_request_review", &e)
                .unwrap()
                .unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/polybar] Pull request reviewed: #3 chore: Configure Renovate"
            );
            assert_eq!(msg["embeds"][0]["description"].as_str().unwrap(), "normal");
        }
    }

    mod membership {
        use crate::{make_discord_message, Event};

        #[test]
        fn added() {
            let payload = include_str!("../fixtures/membership/added.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("membership", &e).unwrap().unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/staff] Member backwardspy added"
            );
        }

        #[test]
        fn removed() {
            let payload = include_str!("../fixtures/membership/removed.json");
            let e: Event = serde_json::from_str(payload).unwrap();
            let msg = make_discord_message("membership", &e).unwrap().unwrap();
            assert_eq!(
                msg["embeds"][0]["title"].as_str().unwrap(),
                "[catppuccin-rfc/staff] Member backwardspy removed"
            );
        }
    }
}
