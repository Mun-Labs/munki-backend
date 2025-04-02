use sqlx::PgPool;

pub async fn last_active(pool: &PgPool, token: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE token_watch SET last_active = NOW() WHERE token_address = any($1)")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

