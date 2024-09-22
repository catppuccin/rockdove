use octocrab::models::webhook_events::{
    payload::{ReleaseWebhookEventAction, ReleaseWebhookEventPayload},
    WebhookEvent,
};

use crate::{colors::RELEASE_COLOR, embed_builder::EmbedBuilder};

pub fn make_release_embed(
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

#[cfg(test)]
mod tests {
    use crate::{
        make_embed,
        tests::{embed_context, TestConfig},
    };

    #[test]
    fn released() {
        let payload = include_str!("../../fixtures/release/released.json");
        let TestConfig {
            webhook_event: event,
            mut settings,
        } = TestConfig::new("release", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
