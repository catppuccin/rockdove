use octocrab::models::webhook_events::{
    payload::{RepositoryWebhookEventAction, RepositoryWebhookEventPayload},
    WebhookEvent,
};

use crate::{
    colors::{COLORS, REPO_COLOR},
    embed_builder::EmbedBuilder,
    errors::{RockdoveError, RockdoveResult},
};

pub fn make_embed(
    event: WebhookEvent,
    specifics: &RepositoryWebhookEventPayload,
) -> RockdoveResult<Option<EmbedBuilder>> {
    let repo = event
        .repository
        .ok_or_else(|| RockdoveError::MissingField {
            event_type: event.kind.clone(),
            field: "repository",
        })?;

    let mut embed = EmbedBuilder::default();

    let full_name = repo.full_name.as_ref().unwrap_or(&repo.name);

    embed.title(&format!(
        "[{}] Repository {}",
        full_name,
        match specifics.action {
            RepositoryWebhookEventAction::Archived => "archived".to_string(),
            RepositoryWebhookEventAction::Created => "created".to_string(),
            RepositoryWebhookEventAction::Deleted => "deleted".to_string(),
            RepositoryWebhookEventAction::Renamed => {
                format!(
                    "renamed from {} to {}",
                    specifics
                        .changes
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "changes",
                        })?
                        .repository
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "changes.repository",
                        })?
                        .name
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "changes.repository.name",
                        })?
                        .from,
                    repo.name,
                )
            }
            RepositoryWebhookEventAction::Transferred => {
                format!(
                    "transferred from {} to {}",
                    specifics
                        .changes
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "changes",
                        })?
                        .owner
                        .as_ref()
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "changes.owner",
                        })?
                        .from
                        .user
                        .login,
                    repo.owner
                        .ok_or_else(|| RockdoveError::MissingField {
                            event_type: event.kind.clone(),
                            field: "repository.owner",
                        })?
                        .login
                )
            }
            RepositoryWebhookEventAction::Unarchived => "unarchived".to_string(),
            _ => {
                return Ok(None);
            }
        }
    ));

    embed.url(
        repo.html_url
            .ok_or_else(|| RockdoveError::MissingField {
                event_type: event.kind.clone(),
                field: "repository.html_url",
            })?
            .as_str(),
    );

    embed.color(match specifics.action {
        RepositoryWebhookEventAction::Deleted => COLORS.red,
        RepositoryWebhookEventAction::Transferred => COLORS.pink,
        _ => REPO_COLOR,
    });

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
      created = { "created" },
      renamed = { "renamed" },
      transferred = { "transferred" },
    )]
    fn snapshot(event_type: &str) {
        let event = "repository";
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
