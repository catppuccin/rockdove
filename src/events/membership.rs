use octocrab::models::webhook_events::{
    payload::{MembershipWebhookEventAction, MembershipWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::MEMBERSHIP_COLOR,
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &MembershipWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    let team_name = specifics
        .team
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "team.name",
        })?;

    let member_login = specifics
        .member
        .get("login")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "member.login",
        })?;

    let mut embed = EmbedBuilder::default();

    embed.title(&format!(
        "[{}] {} {} {} team",
        event
            .organization
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "organization"
            })?
            .login,
        member_login,
        match specifics.action {
            MembershipWebhookEventAction::Added => "added to",
            MembershipWebhookEventAction::Removed => "removed from",
            _ => {
                return Ok(None);
            }
        },
        team_name
    ));

    embed.url(
        specifics
            .team
            .get("html_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "team.html_url",
            })?,
    );

    embed.color(MEMBERSHIP_COLOR);

    Ok(Some(embed))
}

#[cfg(test)]
mod tests {
    use crate::{
        events::make_embed,
        tests::{embed_context, TestConfig},
    };
    use std::fs;
    use yare::parameterized;

    #[parameterized(
        added = { "added" },
        removed = { "removed" },
      )]
    fn snapshot(event_type: &str) {
        let event = "membership";
        let root = env!("CARGO_MANIFEST_DIR");
        let filename = format!("{root}/fixtures/{event}/{event_type}.json");
        let payload = fs::read_to_string(&filename).expect("fixture exists");
        let TestConfig {
            webhook_event,
            mut settings,
        } = TestConfig::new(event, &payload);

        let embed = make_embed(webhook_event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
