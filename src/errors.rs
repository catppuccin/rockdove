use octocrab::models::webhook_events::WebhookEventType;
use thiserror::Error;

use crate::embed_builder;

#[derive(Debug, Error)]
pub enum RockdoveError {
    #[error("missing field in event: {event_type:?}::{field}")]
    MissingField {
        event_type: WebhookEventType,
        field: &'static str,
    },

    #[error("invalid field in event: {event_type:?}::{field}")]
    InvalidField {
        event_type: WebhookEventType,
        field: &'static str,
    },

    #[error(transparent)]
    EmbedBuilder(#[from] embed_builder::Error),
}

pub type RockdoveResult<T> = Result<T, RockdoveError>;
