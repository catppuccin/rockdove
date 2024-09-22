use octocrab::models::webhook_events::{
    payload::{ReleaseWebhookEventAction, ReleaseWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::RELEASE_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &ReleaseWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    if !matches!(specifics.action, ReleaseWebhookEventAction::Released) {
        return Ok(None);
    }

    let repo = event
        .repository
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "repository",
        })?;

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
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "release.html_url",
            })?,
    );

    if let Some(body) = specifics.release.get("body").and_then(|v| v.as_str()) {
        embed.description(body);
    }

    embed.color(RELEASE_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::snapshot_test;

    #[test]
    fn released() {
        snapshot_test!("release", "released");
    }
}
