mod crud;
use std::sync::Arc;

use axum::{
    extract::{Form, MatchedPath, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use blog_model::{BlogData, BlogId, BlogPost};
use blog_model::{BlogPostPreview, Page};
use sqlx::PgPool;

pub(crate) enum Error {
    Crud,
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::Crud
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Aww crud!").into_response()
    }
}

pub struct BlogService {
    crud: crud::BlogCrud,
}

impl BlogService {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        Self {
            crud: crud::BlogCrud::new(pg_pool),
        }
    }
    pub fn routes(self) -> Router {
        let get_b = Arc::new(self);
        let get_page_b = get_b.clone();
        let put_page_b = get_b.clone();
        let post_page_b = get_b.clone();
        let delete_b = get_b.clone();
        Router::new()
            .route(
                "/posts",
                get(|Query(page): Query<Page>| async move { get_b.get_page(page).await }),
            )
            .route(
                "/posts",
                put(|Json(blog_data): Json<BlogData>| async move {
                    put_page_b.put_post(blog_data).await
                }),
            )
            .route(
                "/posts/:id",
                get(|Path(id): Path<BlogId>| async move { get_page_b.get_post(id).await }),
            )
            .route(
                "/posts/:id",
                post(
                    |Path(id): Path<BlogId>, Json(data): Json<BlogData>| async move {
                        let post = BlogPost { id, data };
                        post_page_b.update(post).await
                    },
                ),
            )
            .route(
                "/posts/:id",
                delete(|Path(id): Path<BlogId>| async move { delete_b.delete(id).await }),
            )
    }

    async fn get_page(&self, page: Page) -> Result<Json<Vec<BlogPostPreview>>, Error> {
        self.crud
            .get_page(page.offset_limit())
            .await
            .map(|ok| ok.into())
            .map_err(Error::from)
    }

    async fn get_post(&self, id: BlogId) -> Result<Json<BlogPost>, Error> {
        self.crud
            .get_post(id)
            .await
            .map(|ok| ok.into())
            .map_err(Error::from)
    }

    async fn put_post(&self, blog_data: BlogData) -> Result<Json<BlogId>, Error> {
        self.crud
            .create_post(&blog_data)
            .await
            .map(|ok| ok.into())
            .map_err(Error::from)
    }

    async fn update(&self, blog_post: BlogPost) -> Result<Json<BlogId>, Error> {
        self.crud
            .update_post(&blog_post)
            .await
            .map(|ok| ok.into())
            .map_err(Error::from)
    }

    async fn delete(&self, id: BlogId) -> Result<(), Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;
    use blog_model::Page;
}
