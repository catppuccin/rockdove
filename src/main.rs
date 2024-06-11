use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    routing::post,
    Router,
};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};

#[derive(serde::Deserialize)]
struct Config {
    github_webhook_secret: String,
    discord_webhook: String,
    discord_bot_webhook: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 {
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
        .with_max_level(Level::INFO)
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
    GithubEvent(e): GithubEvent<serde_json::Value>,
) {
    match hook_target(&e) {
        HookTarget::Normal => {
            info!("sending normal hook");
            send_hook(&e, &app_state.discord_hooks.normal).await;
        }
        HookTarget::Bot => {
            info!("sending bot hook");
            send_hook(&e, &app_state.discord_hooks.bot).await;
        }
        HookTarget::None => info!("ignoring event"),
    }
}

async fn send_hook(e: &serde_json::Value, hook: &str) {
    match reqwest::Client::new().post(hook).json(e).send().await {
        Err(e) => error!(?e, "failed to send hook"),
        Ok(r) => info!(?r, "hook sent"),
    };
}

fn hook_target(e: &serde_json::Value) -> HookTarget {
    let sender_type = e.pointer("/payload/sender/type").and_then(|v| v.as_str());
    let private = e
        .pointer("/payload/repository/private")
        .and_then(|v| v.as_bool());

    if sender_type == Some("Bot") {
        return HookTarget::Bot;
    }

    if private == Some(true) {
        info!("ignoring private repository event");
        return HookTarget::None;
    }

    HookTarget::Normal
}
