use anyhow::Context;
use async_trait::async_trait;
use sqlx::{query, query_as};
use sqlx::types::time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

use crate::database::user::User;
use crate::database::Database;

use super::{Session, SessionsRepository};

#[async_trait]
impl SessionsRepository for Database {
    async fn new_session(
        &self,
        user_id: Uuid,
        user_agent: &str,
        exp: &OffsetDateTime,
    ) -> anyhow::Result<Session> {
        query_as::<_, Session>(
            r#"
        insert into sessions (id, user_id, user_agent, exp)
        values (?, ?, ?, ?)
        returning *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(user_agent)
        .bind(exp.to_offset(UtcOffset::UTC))
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating a session")
    }

    async fn get_user_by_session_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        // exp is stored as RFC 3339 text in UTC, so a string comparison orders it correctly
        query_as::<_, User>(
            r#"
        select users.* from users
        inner join sessions
        on users.id = sessions.user_id
        where sessions.exp >= ? and sessions.id = ?
            "#,
        )
        .bind(OffsetDateTime::now_utc())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("user was not found")
    }

    async fn delete_session(&self, id: Uuid) -> anyhow::Result<()> {
        query("delete from sessions where id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("an unexpected error occurred deleting the session")?;

        Ok(())
    }
}
