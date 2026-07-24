use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

pub struct ReactionRow {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
}

pub async fn insert_reaction(
    pool: &PgPool,
    id: Uuid,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO timeline_reactions (id, entry_id, user_id, emoji, created_at)
        VALUES ($1, $2, $3, $4, now())
        "#,
        id,
        entry_id,
        user_id,
        emoji,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e
            && db_err.constraint().is_some()
        {
            return AppError::Conflict("you already reacted with this emoji".into());
        }
        AppError::from(e)
    })?;

    Ok(())
}

pub async fn delete_reaction(
    pool: &PgPool,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM timeline_reactions
        WHERE entry_id = $1 AND user_id = $2 AND emoji = $3
        "#,
        entry_id,
        user_id,
        emoji,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_reactions_for_incident(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<Vec<ReactionRow>, AppError> {
    let rows = sqlx::query_as!(
        ReactionRow,
        r#"
        SELECT r.id, r.entry_id, r.user_id, r.emoji
        FROM timeline_reactions r
        JOIN timeline_entries e ON e.id = r.entry_id
        WHERE e.incident_id = $1
        ORDER BY r.created_at ASC
        "#,
        incident_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
