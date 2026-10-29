use anyhow::Context;
use async_trait::async_trait;
use sqlx::query_as;
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use crate::database::Database;

use super::{User, UsersRepository};

#[async_trait]
impl UsersRepository for Database {
    async fn create_user(
        &self,
        email: &str,
        name: &str,
        hash_password: &str,
    ) -> anyhow::Result<User> {
        let now = OffsetDateTime::now_utc();
        query_as::<_, User>(
            r#"
        insert into users (id, created_at, updated_at, name, email, password)
        values (?, ?, ?, ?, ?, ?)
        returning *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(now)
        .bind(now)
        .bind(name)
        .bind(email)
        .bind(hash_password)
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the user")
    }

    async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        query_as::<_, User>(
            r#"
        select *
        from users
        where email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("unexpected error while querying for user by email")
    }

    async fn get_user_by_id(&self, id: Uuid) -> anyhow::Result<User> {
        query_as::<_, User>(
            r#"
        select *
        from users
        where id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("user was not found")
    }

    async fn update_user(
        &self,
        id: Uuid,
        email: String,
        name: String,
        password: String,
    ) -> anyhow::Result<User> {
        query_as::<_, User>(
            r#"
        update users
        set
            name = ?,
            email = ?,
            password = ?,
            updated_at = ?
        where id = ?
        returning *
            "#,
        )
        .bind(name)
        .bind(email)
        .bind(password)
        .bind(OffsetDateTime::now_utc())
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("could not update the user")
    }
}
