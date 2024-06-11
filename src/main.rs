use std::{env, sync::Arc};

use anyhow::Context;
use axum::{routing::post, Router};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    let token = env::var("GITHUB_TOKEN").context("failed to get GITHUB_TOKEN")?;
    let discord_webhook = env::var("DISCORD_WEBHOOK").context("failed to get DISCORD_WEBHOOK")?;

    let app = Router::new()
        .route(
            "/webhook",
            post(move |e| webhook(e, discord_webhook.clone())),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(GithubToken(Arc::new(token)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let addr = listener.local_addr()?;
    info!(?addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}

async fn webhook(GithubEvent(e): GithubEvent<serde_json::Value>, discord_webhook: String) {
    if !should_forward(&e) {
        return;
    }

    match reqwest::Client::new()
        .post(discord_webhook)
        .json(&e)
        .send()
        .await
    {
        Err(e) => error!(?e, "failed to send discord webhook"),
        Ok(r) => info!(?r, "discord webhook sent"),
    };
}

fn should_forward(e: &serde_json::Value) -> bool {
    let sender_type = e.pointer("/payload/sender/type").and_then(|v| v.as_str());
    let private = e
        .pointer("/payload/repository/private")
        .and_then(|v| v.as_bool());

    if sender_type == Some("Bot") {
        info!("ignoring bot event");
        return false;
    }

    if private == Some(true) {
        info!("ignoring private repository event");
        return false;
    }

    true
}
