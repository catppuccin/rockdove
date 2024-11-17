use std::sync::Arc;

mod events;

use axum::{
    extract::{FromRef, State},
    http::HeaderMap,
    routing::post,
    Router,
};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use colors::COLORS;
use embed_builder::EmbedBuilder;
use errors::RockdoveError;
use octocrab::models::{webhook_events::WebhookEvent, Author};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};

mod colors;
mod embed_builder;
mod errors;

#[derive(serde::Deserialize)]
struct Config {
    github_webhook_secret: String,
    discord_webhook: String,
    discord_bot_webhook: String,
    discord_userstyles_webhook: String,
    discord_error_webhook: String,
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
    userstyles: String,
    error: String,
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
                userstyles: config.discord_userstyles_webhook,
                error: config.discord_error_webhook,
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
    Userstyles,
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
        HookTarget::Userstyles => {
            info!("hook target is userstyles");
            &app_state.discord_hooks.userstyles
        }
        HookTarget::None => {
            info!("no target - ignoring event");
            return;
        }
    };

    match events::make_embed(event) {
        Ok(Some(msg)) => send_hook(&msg, hook).await,
        Ok(None) => info!("no embed created - ignoring event"),
        Err(e) => {
            error!(%e, "failed to make discord message");
            send_error_hook(&e, &app_state.discord_hooks.error).await;
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
        // userstyles is a monorepo with a lot of activity so we're sending it to a different discord channel.
        if repository.name == "userstyles" {
            return HookTarget::Userstyles;
        }

        if repository.private.unwrap_or(false) {
            info!("ignoring private repository event");
            return HookTarget::None;
        }
    }

    HookTarget::Normal
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

async fn send_error_hook(e: &RockdoveError, hook: &str) {
    let mut embed = EmbedBuilder::default();
    embed.title("Error");
    embed.description(&e.to_string());
    embed.color(COLORS.red);
    embed.author(make_hammy());
    embed.url("https://goudham.com/");
    let msg = embed
        .try_build()
        .expect("error embed should always be valid");
    send_hook(&msg, hook).await;
}

fn make_hammy() -> Author {
    serde_json::from_value(serde_json::json!({
      "login": "sgoudham",
      "id": 58_985_301,
      "node_id": "MDQ6VXNlcjU4OTg1MzAx",
      "avatar_url": "https://avatars.githubusercontent.com/u/58985301?v=4",
      "gravatar_id": "",
      "url": "https://api.github.com/users/sgoudham",
      "html_url": "https://github.com/sgoudham",
      "followers_url": "https://api.github.com/users/sgoudham/followers",
      "following_url": "https://api.github.com/users/sgoudham/following{/other_user}",
      "gists_url": "https://api.github.com/users/sgoudham/gists{/gist_id}",
      "starred_url": "https://api.github.com/users/sgoudham/starred{/owner}{/repo}",
      "subscriptions_url": "https://api.github.com/users/sgoudham/subscriptions",
      "organizations_url": "https://api.github.com/users/sgoudham/orgs",
      "repos_url": "https://api.github.com/users/sgoudham/repos",
      "events_url": "https://api.github.com/users/sgoudham/events{/privacy}",
      "received_events_url": "https://api.github.com/users/sgoudham/received_events",
      "type": "User",
      "site_admin": false
    }))
    .expect("hammy is always valid :pepe_heart:")
}

#[cfg(test)]
mod tests {
    use octocrab::models::webhook_events::WebhookEvent;
    use serde_json::json;

    pub struct TestConfig {
        pub webhook_event: WebhookEvent,
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
            Self {
                webhook_event: event,
                settings,
            }
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

    #[macro_export]
    macro_rules! snapshot_test {
        ($event:literal, $event_type:expr) => {
            let filename = format!(
                "{}/fixtures/{}/{}.json",
                env!("CARGO_MANIFEST_DIR"),
                $event,
                $event_type
            );
            let payload = std::fs::read_to_string(&filename).expect("fixture exists");
            let $crate::tests::TestConfig {
                webhook_event,
                mut settings,
            } = $crate::tests::TestConfig::new($event, &payload);

            let embed = $crate::events::make_embed(webhook_event)
                .expect("make_embed should succeed")
                .expect("event fixture can be turned into an embed");

            settings.set_info(&$crate::tests::embed_context(&embed));
            settings.bind(|| insta::assert_yaml_snapshot!(embed));
        };
    }
}
