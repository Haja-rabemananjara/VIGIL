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

pub async fn update_release_status(
    pool: &PgPool,
    release_id: Uuid,
    new_status: &str,
) -> sqlx::Result<ReleaseRow> {
    let row = match new_status {
        "in_progress" => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                UPDATE releases
                SET status = $2, started_at = now(), updated_at = now()
                WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(release_id)
            .bind(new_status)
            .fetch_one(pool)
            .await?
        }
        "completed" => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                UPDATE releases
                SET status = $2, completed_at = now(), updated_at = now()
                WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(release_id)
            .bind(new_status)
            .fetch_one(pool)
            .await?
        }
        "cancelled" => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                UPDATE releases
                SET status = $2, cancelled_at = now(), updated_at = now()
                WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(release_id)
            .bind(new_status)
            .fetch_one(pool)
            .await?
        }
        _ => {
            sqlx::query_as::<_, ReleaseRow>(
                r#"
                UPDATE releases
                SET status = $2, updated_at = now()
                WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(release_id)
            .bind(new_status)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(row)
}

pub async fn get_step_by_id(pool: &PgPool, step_id: Uuid) -> sqlx::Result<Option<ReleaseStepRow>> {
    sqlx::query_as::<_, ReleaseStepRow>(r#"SELECT * FROM release_steps WHERE id = $1"#)
        .bind(step_id)
        .fetch_optional(pool)
        .await
}

pub async fn validate_step(
    pool: &PgPool,
    step_id: Uuid,
    validated_by: Uuid,
) -> sqlx::Result<ReleaseStepRow> {
    sqlx::query_as::<_, ReleaseStepRow>(
        r#"
        UPDATE release_steps
        SET validated_by = $2, validated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(step_id)
    .bind(validated_by)
    .fetch_one(pool)
    .await
}

pub async fn create_incident_link(
    pool: &PgPool,
    id: Uuid,
    release_id: Uuid,
    incident_id: Uuid,
    linked_by: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO release_incident_links (id, release_id, incident_id, linked_by)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(release_id)
    .bind(incident_id)
    .bind(linked_by)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_incident_link(
    pool: &PgPool,
    release_id: Uuid,
    incident_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query(
        r#"
        UPDATE release_incident_links
        SET status = 'removed'
        WHERE release_id = $1 AND incident_id = $2 AND status = 'active'
        "#,
    )
    .bind(release_id)
    .bind(incident_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn has_active_link(
    pool: &PgPool,
    release_id: Uuid,
    incident_id: Uuid,
) -> sqlx::Result<bool> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM release_incident_links
        WHERE release_id = $1 AND incident_id = $2 AND status = 'active'
        "#,
    )
    .bind(release_id)
    .bind(incident_id)
    .fetch_one(pool)
    .await?;

    Ok(row > 0)
}

pub async fn count_active_unresolved_links(pool: &PgPool, release_id: Uuid) -> sqlx::Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM release_incident_links ril
        JOIN incidents i ON i.id = ril.incident_id
        WHERE ril.release_id = $1
          AND ril.status = 'active'
          AND i.status != 'resolved'
        "#,
    )
    .bind(release_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

pub async fn get_blocked_releases_for_incident(
    pool: &PgPool,
    incident_id: Uuid,
) -> sqlx::Result<Vec<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT DISTINCT r.*
        FROM releases r
        JOIN release_incident_links ril ON ril.release_id = r.id
        WHERE ril.incident_id = $1
          AND ril.status = 'active'
          AND r.status = 'blocked'
        "#,
    )
    .bind(incident_id)
    .fetch_all(pool)
    .await
}
