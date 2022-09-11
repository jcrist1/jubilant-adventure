mod post;
mod stream;

use blog_model::{BlogData, BlogId, BlogPost, Page};
use post::PostComponent;
use stream::StreamView;
use sycamore::{prelude::*, suspense::Suspense};

pub(crate) const LOADING: &str = "Loading...";

#[derive(Clone, Copy)]
pub enum BlogView {
    Stream(Page),
    PostView(BlogId, Page, bool),
}

impl BlogView {
    pub fn new() -> Self {
        Self::Stream(Page::new(0))
    }
}

#[derive(Prop)]
struct BlogPostProps<'a> {
    id: BlogId,
    blog_view: &'a Signal<BlogView>,
    last_page: Page,
}

#[derive(Prop)]
pub struct BlogProps {
    class: String,
}

#[component]
#[allow(non_snake_case)]
pub async fn Blog<G: Html>(cx: Scope<'_>, BlogProps { class }: BlogProps) -> View<G> {
    let blog_view = create_signal(cx, BlogView::new());
    view! {cx,
        div {({
            match *blog_view.get().as_ref() {
                BlogView::Stream(page) => {
                    view! {cx,
                        StreamView(
                            current_page=page,
                            blog_view=blog_view
                        )
                    }
                }
                BlogView::PostView(blog_id, last_page, editing) => {
                    view! { cx,
                        Suspense(fallback= view! {cx, (LOADING)}) {
                            PostComponent(
                                blog_id=blog_id,
                                last_page=last_page,
                                blog_view=blog_view,
                                editing = editing
                            )
                        }
                    }
                }
            }
        })}
    }
}

#[cfg(test)]
mod tests {
    use blog_model::Page;

    #[test]
    fn encode_page() {
        let page = Page::new(3);
        let encoded = serde_urlencoded::to_string(page).unwrap();
        println!("{encoded}");
        panic!("BOO")
    }
}
