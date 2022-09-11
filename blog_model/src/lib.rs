use std::fmt::Display;

use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use sqlx::{FromRow, Postgres, Type};

/// Should guarantee that it is valid markdown and possibly sanitise
#[cfg(not(target_arch = "wasm32"))]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct MarkDown(String);

#[cfg(target_arch = "wasm32")]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarkDown(String);

impl MarkDown {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(dirty_markdown: &str) -> Self {
        MarkDown(dirty_markdown.to_string())
    }

    pub fn sanitized_html(&self) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.0, options);

        // Write to String buffer.
        let mut html_output: String = String::with_capacity(self.0.len() * 2);
        html::push_html(&mut html_output, parser);
        ammonia::clean(&html_output)
    }

    fn truncate(mut self) -> Self {
        Self(self.0[..30].to_string())
    }
}

/// assumptions:
/// * no special character control character. Must be sanitized
/// * max length (tbd)
#[cfg(not(target_arch = "wasm32"))]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct Title(String);

#[cfg(target_arch = "wasm32")]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Title(String);

impl Title {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn new(raw_str: &str) -> Self {
        Title(raw_str.into())
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Default, Hash, Type)]
#[sqlx(transparent)]
pub struct BlogId(i64);

#[cfg(target_arch = "wasm32")]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Default, Hash)]
pub struct BlogId(i64);

#[cfg(not(target_arch = "wasm32"))]
#[derive(FromRow)]
pub struct IdRow<IdType: sqlx::Type<Postgres>> {
    pub id: IdType,
}

impl From<BlogId> for i64 {
    fn from(BlogId(id): BlogId) -> Self {
        id
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct BlogData {
    pub title: Title,
    pub description: MarkDown,
    pub created: time::OffsetDateTime,
    pub updated: time::OffsetDateTime,
}

impl BlogData {
    pub fn new(title: Title, description: MarkDown, created: time::OffsetDateTime) -> Self {
        Self {
            title,
            description,
            created,
            updated: created,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: BlogId,
    pub data: BlogData,
}

impl BlogPost {
    pub fn preview(self) -> BlogPostPreview {
        let BlogPost {
            id,
            data:
                BlogData {
                    title,
                    description,
                    created,
                    updated,
                },
        } = self;
        BlogPostPreview {
            id,
            data: BlogData {
                title,
                description,
                created,
                updated,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct BlogPostPreview {
    pub id: BlogId,
    pub data: BlogData,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(FromRow)]
pub struct BlogRow {
    pub id: BlogId,
    pub title: Title,
    pub description: MarkDown,
    pub created: time::OffsetDateTime,
    pub updated: time::OffsetDateTime,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<BlogRow> for BlogPost {
    fn from(
        BlogRow {
            id,
            title,
            description,
            created,
            updated,
        }: BlogRow,
    ) -> Self {
        Self {
            id,
            data: BlogData {
                title,
                description,
                created,
                updated,
            },
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<BlogPost> for BlogRow {
    fn from(
        BlogPost {
            id,
            data:
                BlogData {
                    title,
                    description,
                    created,
                    updated,
                },
        }: BlogPost,
    ) -> BlogRow {
        BlogRow {
            id,
            title,
            description,
            created,
            updated,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Eq, Debug, Default, Clone, Copy)]
pub struct Page {
    page_offset: usize,
}

// todo: offset is a backend struct
#[derive(PartialEq, Serialize, Deserialize, Eq, Debug)]
pub struct OffsetLimit {
    pub offset: usize,
    pub limit: usize,
}

pub const PAGE_SIZE: usize = 10;

impl Page {
    pub fn new(offset: usize) -> Self {
        Page {
            page_offset: offset,
        }
    }

    pub fn next(&self) -> Self {
        let mut next_page = *self;
        next_page.page_offset += 1;
        next_page
    }

    pub fn prev(&self) -> Self {
        if self.page_offset >= 1 {
            let mut prev_page = *self;
            prev_page.page_offset -= 1;
            prev_page
        } else {
            *self
        }
    }

    // todo: offset is a backend struct
    pub fn offset_limit(&self) -> OffsetLimit {
        OffsetLimit {
            offset: PAGE_SIZE * self.page_offset,
            limit: PAGE_SIZE,
        }
    }
}

impl From<Page> for usize {
    fn from(Page { page_offset }: Page) -> Self {
        page_offset
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::to_string_pretty;

    #[test]
    fn test_serde() {
        let string: String = "Boop".into();
        let title = Title(string.clone());
        assert_eq!(format!("\"{string}\""), to_string_pretty(&title).unwrap())
    }

    #[test]
    fn test_page_offset() {
        let page = Page::new(3);
        assert_eq!(
            page.offset_limit(),
            OffsetLimit {
                offset: 30,
                limit: 10
            }
        );
    }

    #[test]
    fn test_markdown_sanitisation() {
        let html_markdown = r#"# Heading
Somebody once told me the world is gonna roll me.
<script>console.print("YOLO")</script>
"#;
        let clean_html = r#"<h1>Heading</h1>
<p>Somebody once told me the world is gonna roll me.</p>

"#;
        let markdown = MarkDown::new(html_markdown);
        assert_eq!(clean_html, &markdown.sanitized_html());
        let html_code_markdown = r#"# Heading
Somebody once told me the world is gonna roll me.
```html
<script>console.print("YOLO")</script>
```
"#;
        let clean_html = r#"<h1>Heading</h1>
<p>Somebody once told me the world is gonna roll me.</p>
<pre><code>&lt;script&gt;console.print("YOLO")&lt;/script&gt;
</code></pre>
"#;
        let markdown = MarkDown::new(html_code_markdown);
        assert_eq!(clean_html, &markdown.sanitized_html());
    }
}
