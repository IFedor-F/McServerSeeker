use super::ParsedForSqlServerData;
use sqlx::PgPool;
use tokio::sync::mpsc;

pub struct DbQueueWorker {
    db_pool: PgPool,
}

impl DbQueueWorker {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    pub async fn run(self, mut rx: mpsc::Receiver<ParsedForSqlServerData>) {
        while let Some(data) = rx.recv().await {
            let mut tx = match self.db_pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    log::error!("failed to start transaction: {e}");
                    return;
                }
            };

            if let Err(e) = data.write_to_tx(&mut tx).await {
                log::error!("failed to save server: {}", e);
            } else {
                if let Err(e) = tx.commit().await {
                    log::error!("failed to rollback db: {e}");
                }
            }
        }
    }
}
