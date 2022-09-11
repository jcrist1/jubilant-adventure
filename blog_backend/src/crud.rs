use sqlx::{PgPool, Postgres};
use std::sync::Arc;

use blog_model::{BlogData, BlogId, BlogPost, IdRow, MarkDown, Title};
use blog_model::{BlogPostPreview, OffsetLimit};
use futures_util::{StreamExt, TryStreamExt};
use sqlx::prelude::*;
use tracing::info;

pub(crate) struct BlogCrud(Arc<PgPool>);

impl BlogCrud {
    pub(crate) fn new(pool: Arc<PgPool>) -> Self {
        Self(pool)
    }

    pub(crate) async fn get_page(
        &self,
        offset_limit: OffsetLimit,
    ) -> Result<Vec<BlogPostPreview>, ()> {
        let mut ex = self.0.begin().await.map_err(|_| ())?;
        // todo: remove debug
        info!("{offset_limit:?}");
        sqlx::query_as!(
            blog_model::BlogRow,
            r#"
                SELECT 
                    id as "id: BlogId",
                    title as "title: _",
                    description as "description: _",
                    created,
                    updated
                FROM blog_posts
                ORDER BY created DESC
                LIMIT $1
                OFFSET $2
            "#,
            offset_limit.limit as i64,
            offset_limit.offset as i64
        )
        .fetch(&mut ex)
        .map(|res| {
            res.map(BlogPost::from)
                .map(BlogPost::preview)
                .map_err(|_| ())
        })
        .try_collect::<Vec<_>>()
        .await
    }

    pub(crate) async fn get_post(&self, id: BlogId) -> Result<BlogPost, ()> {
        let mut ex = self.0.begin().await.map_err(|_| ())?;

        sqlx::query_as!(
            blog_model::BlogRow,
            r#"
                SELECT 
                    id as "id:_",
                    title as "title: _",
                    description as "description: _",
                    created,
                    updated
                FROM blog_posts
                WHERE id = $1
            "#,
            i64::from(id)
        )
        .fetch_one(&mut ex)
        // todo: error handling
        .await
        .map(BlogPost::from)
        .map_err(|err| println!("Got err: {err:?}"))
    }

    pub(crate) async fn create_post(
        &self,
        BlogData {
            title,
            description,
            created,
            updated,
        }: &BlogData,
    ) -> Result<BlogId, ()> {
        let mut ex = self.0.begin().await.map_err(|_| ())?;
        let id = sqlx::query_as!(
            IdRow::<BlogId>,
            r#"
                INSERT INTO blog_posts (
                    title,
                    description,
                    created,
                    updated
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                ) RETURNING id as "id: _"
            "#,
            title.as_str(),
            description.as_str(),
            created,
            updated,
        )
        .fetch_one(&mut ex)
        .await
        .map_err(|_| ());
        ex.commit().await.map_err(|_| {})?;
        id.map(|id| id.id)
    }

    pub(crate) async fn update_post(
        &self,
        BlogPost {
            id,
            data:
                BlogData {
                    title,
                    description,
                    created,
                    updated,
                },
        }: &BlogPost,
    ) -> Result<BlogId, ()> {
        let mut ex = self.0.begin().await.map_err(|_| ())?;
        let id = sqlx::query_as!(
            IdRow::<BlogId>,
            r#"
                UPDATE blog_posts SET 
                    title = $2,
                    description = $3,
                    created = $4,
                    updated = $5
                WHERE id = $1
                RETURNING id as "id: _"
            "#,
            i64::from(*id),
            title.as_str(),
            description.as_str(),
            created,
            updated,
        )
        .fetch_one(&mut ex)
        .await
        .map_err(|_| ());
        ex.commit().await.map_err(|_| {})?;
        id.map(|id| id.id)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use blog_model::{BlogData, BlogPost, MarkDown, Title};
    use sqlx::PgPool;
    use time::OffsetDateTime;

    use super::BlogCrud;

    #[test]
    fn test_write_and_get_post() -> Result<(), ()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        let fut = async {
            let pool = PgPool::connect("postgresql://blog_user:secret123@localhost:5436/blog")
                .await
                .unwrap();
            let blog_crud = BlogCrud(Arc::new(pool));
            let created = OffsetDateTime::now_utc();
            let blog_data = BlogData::new(Title::default(), MarkDown::default(), created);
            let id = blog_crud.create_post(&blog_data).await.unwrap();
            println!("ID: {id:?}");
            let blog_post = blog_crud.get_post(id).await.unwrap();
            println!("{blog_post:?}");
            assert_eq!(
                blog_post,
                BlogPost {
                    id,
                    data: blog_data,
                },
            );
            let new_data = BlogData::new(
                Title::new("escaped"),
                MarkDown::new("Blloop"),
                time::OffsetDateTime::now_utc(),
            );
            let post = BlogPost { id, data: new_data };
            let updated_id = blog_crud.update_post(&post).await.unwrap();
            let updated_post = blog_crud.get_post(id).await.unwrap();
            assert_eq!(post, updated_post);
        };
        // todo: put and test retrieval
        runtime.block_on(async { fut.await });
        Ok(())
    }
}
