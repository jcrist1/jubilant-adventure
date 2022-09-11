use blog_frontend::Blog;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

#[allow(non_snake_case)]
#[component]
async fn App<G: Html>(ctx: Scope<'_>) -> View<G> {
    view! { ctx,
        div { "Hi there!" }
    }
}

enum Apps {
    Blog,
    Recipes,
    Showcase,
}

struct AppState {
    selected: Apps,
    recipe_state: (),
    blog_state: (),
    showcase_state: (),
}

fn main() {
    console_error_panic_hook::set_once();

    // Add this line:
    tracing_wasm::set_as_global_default();

    sycamore::render(|ctx| {
        view! { ctx,
            Blog(class="boo".to_string())
        }
    });
}
