use octocrab::models::webhook_events::{
    payload::{RepositoryWebhookEventAction, RepositoryWebhookEventPayload},
    WebhookEvent,
};

use crate::{embed_builder::EmbedBuilder, COLORS};

const REPO_COLOR: catppuccin::Color = COLORS.yellow;

pub fn make_repository_embed(
    event: WebhookEvent,
    specifics: &RepositoryWebhookEventPayload,
) -> Option<EmbedBuilder> {
    let repo = event
        .repository
        .expect("repository events should always have a repository");

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
                        .expect("repository renamed event should always have changes")
                        .repository
                        .as_ref()
                        .expect("repository renamed event changes should always have a repository")
                        .name
                        .as_ref()
                        .expect("repository renamed event changes should always have a name")
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
                        .expect("repository transferred event should always have changes")
                        .owner
                        .as_ref()
                        .expect("repository transferred event changes should always have an owner")
                        .from
                        .user
                        .login,
                    repo.owner
                        .expect("repository should always have an owner")
                        .login
                )
            }
            RepositoryWebhookEventAction::Unarchived => "unarchived".to_string(),
            _ => {
                return None;
            }
        }
    ));

    embed.url(
        repo.html_url
            .expect("repository should always have an html url")
            .as_str(),
    );

    embed.color(match specifics.action {
        RepositoryWebhookEventAction::Deleted => COLORS.red,
        RepositoryWebhookEventAction::Transferred => COLORS.pink,
        _ => REPO_COLOR,
    });

    Some(embed)
}

#[cfg(test)]
mod tests {
    use crate::{
        make_embed,
        tests::{embed_context, TestConfig},
    };

    #[test]
    fn created() {
        let payload = include_str!("../../fixtures/repository/created.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("repository", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn renamed() {
        let payload = include_str!("../../fixtures/repository/renamed.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("repository", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }

    #[test]
    fn transferred() {
        let payload = include_str!("../../fixtures/repository/transferred.json");
        let TestConfig {
            event,
            mut settings,
        } = TestConfig::new("repository", payload);

        let embed = make_embed(event)
            .expect("make_embed should succeed")
            .expect("event fixture can be turned into an embed");

        settings.set_info(&embed_context(&embed));
        settings.bind(|| insta::assert_yaml_snapshot!(embed));
    }
}
