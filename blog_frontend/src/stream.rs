use crate::LOADING;
use blog_model::{BlogData, BlogId, BlogPost, MarkDown, Page, Title};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use tracing::warn;

use crate::BlogView;

pub(crate) async fn get_posts(page_offset: Page) -> anyhow::Result<Vec<BlogPost>> {
    let offset_param = serde_urlencoded::to_string(page_offset)?;
    reqwasm::http::Request::get(&format!("/api/v1/blog/posts?{offset_param}"))
        .send()
        .await
        .map_err(anyhow::Error::from)?
        .json()
        .await
        .map_err(anyhow::Error::from)
}

#[derive(Prop)]
pub(crate) struct StreamProps<'a> {
    pub(crate) current_offset: Page,
    pub(crate) blog_view: &'a Signal<BlogView>,
}
async fn create_post<'a>() -> Result<BlogId, anyhow::Error> {
    let blog_data = BlogData::new(
        Title::new(""),
        MarkDown::new(""),
        time::OffsetDateTime::now_utc(),
    );
    let body = serde_json::to_string(&blog_data)?;
    let new_post = reqwasm::http::Request::put("api/v1/blog/posts")
        .body(body)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<BlogId>()
        .await?;
    Ok(new_post)
}

#[component]
#[allow(non_snake_case)]
pub(crate) async fn StreamComponent<'a, G: Html>(
    cx: Scope<'a>,
    StreamProps {
        current_offset,
        blog_view,
    }: StreamProps<'a>,
) -> View<G> {
    let posts_res = get_posts(current_offset).await;
    match posts_res {
        Err(err) => {
            warn!("Failed to load page of blog posts with offset {current_offset:?}: {err:?}");
            view! { cx,
                "Error: Unable to load posts"
            }
        }
        Ok(posts) => {
            let posts = create_signal(cx, posts);
            view!(
                cx,
                div {(format!("{}", usize::from(current_offset)))}
                div(on:click = move |_| {
                    spawn_local_scoped(cx, async move {
                        match create_post().await {
                            Ok(new_id) => blog_view.set(BlogView::PostView(new_id, current_offset, true)),
                            Err(err) => warn!("failed to create post: {err:?}"),
                        }
                    })
                }) { "+" }
                Keyed(
                    iterable = posts,
                    view = move |cx, BlogPost { data, id }| {
                        view! { cx,
                            div(on:click= move |_| {
                                blog_view.set(BlogView::PostView(id, current_offset, false));
                            }) {
                                BlogPostPreview(class = format!(""), post = data)
                            }
                        }
                    },
                    key = |post| post.id
                )
            )
        }
    }
}

#[derive(Prop)]
pub(crate) struct StreamViewProps<'a> {
    current_page: Page,
    blog_view: &'a Signal<BlogView>,
}

#[component]
#[allow(non_snake_case)]
pub(crate) fn StreamView<'a, G: Html>(
    cx: Scope<'a>,
    StreamViewProps {
        current_page,
        blog_view,
    }: StreamViewProps<'a>,
) -> View<G> {
    let page = create_signal(cx, current_page);
    view! {cx,
        ({
            let offset = *page.get();
            view! { cx, Suspense(fallback=view! { cx, (LOADING)}) {
                StreamComponent(
                    current_offset=offset,
                    blog_view=blog_view
                )
            }}
        })
        div {
            // todo: last and first
            div(on:click=|_| {
                let next = page.get().prev();
                page.set(next);
            }) { "<" }
            div {(format!("{:?}", *page.get()))}
            div(on:click=|_| {
                let prev = page.get().next();
                page.set(prev);
            }) { ">" }
        }
    }
}

#[derive(Prop)]
struct BlogPostPreviewProps {
    class: String,
    post: BlogData,
}

#[component]
#[allow(non_snake_case)]
fn BlogPostPreview<G: Html>(
    cx: Scope,
    BlogPostPreviewProps { class, post }: BlogPostPreviewProps,
) -> View<G> {
    view!(cx,
        div(class = class) {
            div {(post.title)}
            div {(format!("created: {} – updated: {}", post.created, post.updated))}
            div(dangerously_set_inner_html = &post.description.sanitized_html())
        }
    )
}
