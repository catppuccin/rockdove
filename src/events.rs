use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use tracing::info;

use crate::{
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

mod commit_comment;
mod discussion;
mod discussion_comment;
mod issue_comment;
mod issues;
mod membership;
mod pull_request;
mod pull_request_review;
mod release;
mod repository;

pub fn make_embed(event: WebhookEvent) -> RockdoveResult<Option<serde_json::Value>> {
    let sender = event
        .sender
        .clone()
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "sender",
        })?;

    let Some(mut embed) = begin_embed(event)? else {
        info!("ignoring event");
        return Ok(None);
    };

    embed.author(sender);
    Ok(Some(embed.try_build()?))
}

fn begin_embed(event: WebhookEvent) -> RockdoveResult<Option<EmbedBuilder>> {
    match event.specific.clone() {
        WebhookEventPayload::Repository(specifics) => repository::make_embed(event, &specifics),
        WebhookEventPayload::Discussion(specifics) => discussion::make_embed(event, &specifics),
        WebhookEventPayload::DiscussionComment(specifics) => {
            discussion_comment::make_embed(event, &specifics)
        }
        WebhookEventPayload::Issues(specifics) => issues::make_embed(event, &specifics),
        WebhookEventPayload::PullRequest(specifics) => pull_request::make_embed(event, &specifics),
        WebhookEventPayload::IssueComment(specifics) => {
            issue_comment::make_embed(event, &specifics)
        }
        WebhookEventPayload::CommitComment(specifics) => {
            commit_comment::make_embed(event, &specifics)
        }
        WebhookEventPayload::PullRequestReview(specifics) => {
            pull_request_review::make_embed(event, &specifics)
        }
        WebhookEventPayload::Release(specifics) => release::make_embed(event, &specifics),
        WebhookEventPayload::Membership(specifics) => membership::make_embed(event, &specifics),
        _ => Ok(None),
    }
}
