use crate::BlogView;
use blog_model::{BlogData, BlogId, BlogPost, MarkDown, Page, Title};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use tracing::{info, warn};
use wasm_bindgen::JsCast;
use web_sys::{Event, EventTarget, HtmlInputElement, HtmlTextAreaElement};

pub(crate) async fn get_post(blog_id: BlogId) -> Result<BlogPost, ()> {
    let encoded = i64::from(blog_id);
    reqwasm::http::Request::get(&format!("/api/v1/blog/posts/{encoded}"))
        .send()
        .await
        .map_err(|_| ())?
        .json::<BlogPost>()
        .await
        .map_err(|_| ())
}
#[derive(Prop)]
pub struct PostViewProps<'a> {
    blog_id: BlogId,
    editing: bool,
    last_page: Page,
    blog_view: &'a Signal<BlogView>,
}

#[derive(Clone, Copy, Debug)]
struct BlogPostData<'a> {
    editing: &'a Signal<bool>,
    saved: &'a Signal<bool>,
    id: BlogId,
    blog_data: &'a Signal<BlogData>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),
}

async fn save_post<'a>(
    blog_id: BlogId,
    blog_post_data: BlogPostData<'a>,
) -> Result<(), anyhow::Error> {
    let blog_data = blog_post_data.blog_data.get();
    let body = serde_json::to_string(&blog_data)?;
    reqwasm::http::Request::post(&format!("/api/v1/blog/posts/{}", i64::from(blog_id)))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    Ok(())
}

#[derive(Prop)]
struct PostProps {
    blog_data: BlogPost,
    editing: bool,
}

#[component]
#[allow(non_snake_case)]
fn Post<G: Html>(cx: Scope<'_>, PostProps { blog_data, editing }: PostProps) -> View<G> {
    let blog_post_data = BlogPostData {
        editing: create_signal(cx, editing),
        saved: create_signal(cx, true),
        id: blog_data.id,
        blog_data: create_signal(cx, blog_data.data),
    };
    let set_title = move |event: Event| {
        let get_input = move || -> Result<String, anyhow::Error> {
            let value = event
                .target()
                .ok_or_else(|| Error::Message("No target found".to_string()))?
                .dyn_into::<HtmlInputElement>()
                .map_err(|err| {
                    Error::Message(format!(
                        "Failed to find HtmlInputElement from event target: {err:?}"
                    ))
                })?
                .value();
            Ok(value)
        };
        match get_input() {
            Err(err) => warn!("Failed to set description: {err:?}"),
            Ok(input_value) => {
                let mut data = blog_post_data.blog_data.get().as_ref().clone();
                data.title = Title::new(&input_value);
                data.updated = time::OffsetDateTime::now_utc();
                blog_post_data.blog_data.set(data);
                blog_post_data.saved.set(false);
            }
        };
    };
    let set_description = move |event: Event| {
        let get_input = move || -> Result<String, anyhow::Error> {
            let value = event
                .target()
                .ok_or_else(|| Error::Message("No target found".to_string()))?
                .dyn_into::<HtmlTextAreaElement>()
                .map_err(|err| {
                    Error::Message(format!(
                        "Failed to find HtmlTextAreaElemet from event target: {err:?}"
                    ))
                })?
                .value();
            Ok(value)
        };
        match get_input() {
            Err(err) => warn!("Failed to set description: {err:?}"),
            Ok(input_value) => {
                let mut data = blog_post_data.blog_data.get().as_ref().clone();
                data.description = MarkDown::new(&input_value);
                blog_post_data.blog_data.set(data);
                blog_post_data.saved.set(false);
            }
        };
    };
    let post = blog_post_data.blog_data;
    info!("Hello");
    let id = blog_post_data.id;
    view! { cx, ({
        if *blog_post_data.editing.get() {
            view! { cx,
                div {
                    input(type = "text", on:input = set_title, value=post.get().title.as_str().to_string()) {

                    }
                    textarea(on:input = set_description) {
                        (post.get().description.as_str().to_string())
                    }
                }
                div(on:click = move |_| {
                    blog_post_data.editing.set(false);
                    spawn_local_scoped(cx, async move {
                        match save_post(id, blog_post_data).await {
                            Err(err) => warn!("Failed to save post: {err:?}"),
                            Ok(_) => {
                                blog_post_data.saved.set(true);
                            }
                        }
                    });
                }){
                    "O"
                }
            }
        } else {
            view! { cx,
                div(on:dblclick = move |_| {
                    blog_post_data.editing.set(true);
                    info!("BOO");
                }){
                    div {(post.get().title)}
                    div {(format!("created: {} – updated: {}", post.get().created, post.get().updated))}
                    div(dangerously_set_inner_html = &post.get().description.sanitized_html())
                }
            }
        }
    })}
}

#[component]
#[allow(non_snake_case)]
pub(crate) async fn PostComponent<'a, G: Html>(
    cx: Scope<'a>,
    PostViewProps {
        blog_id,
        editing,
        last_page,
        blog_view,
    }: PostViewProps<'a>,
) -> View<G> {
    let synchronised = create_signal(cx, false);
    let post_view = match get_post(blog_id).await {
        Err(err) => {
            warn!("warning received error: {err:?} while trying to fetch post with");
            view! { cx, "Error in forming request for data"}
        }
        Ok(post) => {
            view! { cx,
                Post(blog_data = post, editing = editing)
            }
        }
    };
    view! {cx,
        (post_view)
        div(on:click=move |_| {
            blog_view.set(BlogView::Stream(last_page))
        }) {"X"}
    }
}
