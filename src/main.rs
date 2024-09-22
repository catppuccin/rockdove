use std::sync::Arc;

mod events;

use axum::{
    extract::{FromRef, State},
    http::HeaderMap,
    routing::post,
    Router,
};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use events::{
    commit_comment::make_commit_comment_embed, issue_comment::make_issue_comment_embed,
    issues::make_issues_embed, membership::make_membership_embed,
    pull_request::make_pull_request_embed, pull_request_review::make_pull_request_review_embed,
    release::make_release_embed, repository::make_repository_embed,
};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};

mod embed_builder;

const COLORS: catppuccin::FlavorColors = catppuccin::PALETTE.mocha.colors;
pub const ISSUE_COLOR: catppuccin::Color = COLORS.green;
pub const PULL_REQUEST_COLOR: catppuccin::Color = COLORS.blue;
pub const REPO_COLOR: catppuccin::Color = COLORS.yellow;
pub const RELEASE_COLOR: catppuccin::Color = COLORS.mauve;
pub const MEMBERSHIP_COLOR: catppuccin::Color = COLORS.base;
pub const COMMIT_COLOR: catppuccin::Color = COLORS.teal;

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
        WebhookEventPayload::Issues(specifics) => make_issues_embed(event, &specifics),
        WebhookEventPayload::PullRequest(specifics) => make_pull_request_embed(event, &specifics),
        WebhookEventPayload::IssueComment(specifics) => make_issue_comment_embed(event, &specifics),
        WebhookEventPayload::CommitComment(specifics) => {
            Some(make_commit_comment_embed(event, &specifics))
        }
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

    pub struct TestConfig {
        pub event: WebhookEvent,
        pub settings: insta::Settings,
    }

    impl TestConfig {
        pub fn new(event_type: &str, payload: &str) -> Self {
            let event = WebhookEvent::try_from_header_and_body(event_type, payload)
                .expect("event fixture is valid");
            let mut settings = insta::Settings::new();
            settings.set_omit_expression(true);
            settings.set_snapshot_path(format!("../../snapshots/{event_type}"));
            settings.set_prepend_module_to_snapshot(false);
            Self { event, settings }
        }
    }

    pub fn embed_context(embed: &serde_json::Value) -> serde_json::Value {
        json!({
            "author_name_length": &embed["embeds"][0]["author"]["name"].as_str().unwrap().len(),
            "title_length": &embed["embeds"][0]["title"].as_str().unwrap().len(),
            "description_length": &embed["embeds"][0]["description"].as_str().unwrap_or("").len(),
            "colour_hex": format!("#{:X}", embed["embeds"][0]["color"].as_u64().unwrap()),
        })
    }
}
