use octocrab::models::Author;
use serde_json::json;

const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 640;
const MAX_AUTHOR_NAME_LENGTH: usize = 256;

#[derive(Default, Debug)]
pub struct EmbedBuilder {
    title: Option<String>,
    url: Option<String>,
    author: Option<Author>,
    description: Option<String>,
    color: Option<u32>,
}

impl EmbedBuilder {
    pub fn title(&mut self, title: &str) -> &Self {
        self.title = Some(limit_text_length(title, MAX_TITLE_LENGTH));
        self
    }

    pub fn url(&mut self, url: &str) -> &Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn author(&mut self, author: Author) -> &Self {
        self.author = Some(author);
        self
    }

    pub fn description(&mut self, description: &str) -> &Self {
        self.description = Some(limit_text_length(description, MAX_DESCRIPTION_LENGTH));
        self
    }

    pub fn color(&mut self, color: catppuccin::Color) -> &Self {
        let rgb = color.rgb;
        self.color = Some(u32::from(rgb.r) << 16 | u32::from(rgb.g) << 8 | u32::from(rgb.b));
        self
    }

    pub fn try_build(self) -> anyhow::Result<serde_json::Value> {
        Ok(json!({
            "embeds": [{
                "title": self.title.ok_or_else(|| anyhow::anyhow!("missing title"))?,
                "url": self.url.ok_or_else(|| anyhow::anyhow!("missing url"))?,
                "description": self.description,
                "color": self.color,
                "author": embed_author(&self.author.ok_or_else(|| anyhow::anyhow!("missing author"))?),
            }],
        }))
    }
}

fn embed_author(author: &Author) -> serde_json::Value {
    json!({
        "name": limit_text_length(&author.login, MAX_AUTHOR_NAME_LENGTH),
        "url": author.html_url,
        "icon_url": author.avatar_url,
    })
}

fn limit_text_length(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        format!("{}...", &text[..max_length - 3])
    } else {
        text.to_string()
    }
}
