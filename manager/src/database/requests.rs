use data_core::proto::scanner as pb;
use ipnetwork::IpNetwork;
use sqlx::{PgPool, Postgres, QueryBuilder};

#[derive(Debug, sqlx::FromRow)]
struct TargetRecord {
    ip: IpNetwork,
    port: i32,
    last_used_nick: Option<String>,
}
pub async fn get_targets(
    db_pool: &PgPool,
    includes: Vec<IpNetwork>,
    excludes: Vec<IpNetwork>,
) -> Result<Vec<pb::RescanTarget>, sqlx::Error> {
    let mut builder: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT ip, port, last_used_nick FROM data.servers WHERE true");

    if !includes.is_empty() {
        builder.push(" AND (ip << ANY(");
        builder.push_bind(includes);
        builder.push("))");
    }
    if !excludes.is_empty() {
        builder.push(" AND NOT (ip << ANY(");
        builder.push_bind(excludes);
        builder.push("))");
    }

    Ok(builder
        .build_query_as::<TargetRecord>()
        .fetch_all(db_pool)
        .await?
        .into_iter()
        .map(|record| pb::RescanTarget {
            ip: Some(record.ip.ip().into()),
            port: record.port as u32,
            player_name: record.last_used_nick,
        })
        .collect())
}
