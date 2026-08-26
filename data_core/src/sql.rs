use sqlx::PgPool;

pub mod data;

pub mod analytics;

pub async fn run_migrations(pool: PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
