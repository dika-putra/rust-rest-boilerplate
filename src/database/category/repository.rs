use anyhow::Context;
use async_trait::async_trait;
use sqlx::types::time::OffsetDateTime;
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::database::Database;

use super::model::{CategoriesRepository, Category, CategoryType};

#[async_trait]
impl CategoriesRepository for Database {
    async fn create_category(
        &self,
        user_id: Uuid,
        name: String,
        cat_type: CategoryType,
    ) -> anyhow::Result<Category> {
        let now = OffsetDateTime::now_utc();
        query_as::<_, Category>(
            r#"
        insert into categories (id, created_at, updated_at, name, user_id, cat_type)
        values (?, ?, ?, ?, ?, ?)
        returning *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(now)
        .bind(now)
        .bind(name)
        .bind(user_id)
        .bind(cat_type)
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the category")
    }

    async fn get_category_by_id(&self, id: Uuid) -> anyhow::Result<Option<Category>> {
        query_as::<_, Category>(
            r#"
        select *
        from categories
        where id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("category was not found")
    }

    async fn get_categories(&self, user_id: Uuid) -> anyhow::Result<Vec<Category>> {
        query_as::<_, Category>(
            r#"
        select *
        from categories
        where user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("category was not found")
    }

    async fn update_category(
        &self,
        id: Uuid,
        name: String,
        cat_type: CategoryType,
    ) -> anyhow::Result<Category> {
        query_as::<_, Category>(
            r#"
        update categories
        set
            name = ?,
            cat_type = ?,
            updated_at = ?
        where id = ?
        returning *
            "#,
        )
        .bind(name)
        .bind(cat_type)
        .bind(OffsetDateTime::now_utc())
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("could not update the category")
    }

    async fn delete_category(&self, id: Uuid) -> anyhow::Result<()> {
        query(
            r#"
        delete from categories
        where id = ?
        "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("an unexpected error occurred deleting category")?;

        Ok(())
    }
}
