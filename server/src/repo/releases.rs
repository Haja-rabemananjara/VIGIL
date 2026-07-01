use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::releases::{ReleaseRow, ReleaseStepRow};

pub async fn create_release(
    pool: &PgPool,
    id: Uuid,
    team_id: Uuid,
    title: &str,
    body: &str,
    created_by: Uuid,
    step_names: &[String],
) -> sqlx::Result<ReleaseRow> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, ReleaseRow>(
        r#"
        INSERT INTO releases (id, team_id, title, body, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(team_id)
    .bind(title)
    .bind(body)
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await?;

    for (i, name) in step_names.iter().enumerate() {
        let step_id = Uuid::new_v4();
        let position = (i + 1) as i32;

        sqlx::query(
            r#"
            INSERT INTO release_steps (id, release_id, name, position)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(step_id)
        .bind(id)
        .bind(name)
        .bind(position)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(row)
}

pub async fn list_releases(
    pool: &PgPool,
    team_id: Uuid,
    status_filter: Option<&str>,
) -> sqlx::Result<Vec<ReleaseRow>> {
    match status_filter {
        Some(status) => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                SELECT * FROM releases
                WHERE team_id = $1 AND status = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(team_id)
            .bind(status)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                SELECT * FROM releases
                WHERE team_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(team_id)
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn get_release_by_id(
    pool: &PgPool,
    release_id: Uuid,
) -> sqlx::Result<Option<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT * FROM releases WHERE id = $1
        "#,
    )
    .bind(release_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_steps_for_release(
    pool: &PgPool,
    release_id: Uuid,
) -> sqlx::Result<Vec<ReleaseStepRow>> {
    sqlx::query_as::<_, ReleaseStepRow>(
        r#"
        SELECT * FROM release_steps
        WHERE release_id = $1
        ORDER BY position ASC
        "#,
    )
    .bind(release_id)
    .fetch_all(pool)
    .await
}

pub async fn get_steps_for_releases(
    pool: &PgPool,
    release_ids: &[Uuid],
) -> sqlx::Result<Vec<ReleaseStepRow>> {
    if release_ids.is_empty() {
        return Ok(vec![]);
    }

    sqlx::query_as::<_, ReleaseStepRow>(
        r#"
        SELECT * FROM release_steps
        WHERE release_id = ANY($1)
        ORDER BY release_id, position ASC
        "#,
    )
    .bind(release_ids)
    .fetch_all(pool)
    .await
}
