use crate::config::DatabaseConfig;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub type DbPool = Pool;

pub fn create_pool(db_config: &DatabaseConfig) -> Result<DbPool, deadpool_postgres::CreatePoolError> {
    let mut cfg = Config::new();
    cfg.user = Some(db_config.user.clone());
    cfg.password = Some(db_config.password.clone());
    cfg.host = Some(db_config.host.clone());
    cfg.port = Some(db_config.port);
    cfg.dbname = Some(db_config.dbname.clone());
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
}
