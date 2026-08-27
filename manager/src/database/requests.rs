use data_core::api::manager::ScanTarget;
use sqlx::PgPool;

pub async fn get_targets(db_pool: &PgPool) -> Result<Vec<ScanTarget>, sqlx::Error> {
    let targets = sqlx::query!("SELECT ip, port, last_used_nick FROM data.servers")
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|record| ScanTarget {
            ip: record.ip.ip(),
            port: record.port as u16,
            player_name: record.last_used_nick,
        })
        .collect();
    Ok(targets)
}
