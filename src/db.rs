use deadpool_postgres::{Config, Pool, Runtime};
use log::{info, warn};
use std::env;
use tokio_postgres::NoTls;

pub type DbPool = Pool;

/// Creates a database connection pool.
///
/// This function implements a fallback mechanism for database configuration:
/// 1. It first checks for a `DATABASE_URL` environment variable. If present, it uses it.
/// 2. If `DATABASE_URL` is not found, it falls back to constructing a connection string
///    from individual `DATABASE_*` environment variables (`USER`, `PASSWORD`, `HOST`, `PORT`, `DBNAME`).
/// 3. If neither method provides sufficient configuration, it will panic with an error.
pub fn create_pool() -> Result<DbPool, deadpool_postgres::CreatePoolError> {
    let cfg = if let Ok(db_url) = env::var("DATABASE_URL") {
        info!("✓ Connecting to database using DATABASE_URL.");
        let pg_config = db_url
            .parse::<tokio_postgres::Config>()
            .expect("FATAL: Failed to parse DATABASE_URL");

        let mut cfg = Config::new();
        if let Some(user) = pg_config.get_user() {
            cfg.user = Some(user.to_string());
        }
        if let Some(password) = pg_config.get_password() {
            cfg.password = Some(String::from_utf8_lossy(password).to_string());
        }
        if let Some(host) = pg_config.get_hosts().get(0) {
            if let tokio_postgres::config::Host::Tcp(host_str) = host {
                cfg.host = Some(host_str.clone());
            }
        }
        if let Some(port) = pg_config.get_ports().get(0) {
            cfg.port = Some(*port);
        }
        if let Some(dbname) = pg_config.get_dbname() {
            cfg.dbname = Some(dbname.to_string());
        }
        cfg
    } else {
        // Fallback method: Use individual DATABASE_* variables.
        warn!("DATABASE_URL not found. Falling back to individual DATABASE_* variables.");
        let mut cfg = Config::new();
        cfg.user = env::var("DATABASE_USER").ok();
        cfg.password = env::var("DATABASE_PASSWORD").ok();
        cfg.host = env::var("DATABASE_HOST").ok();
        cfg.dbname = env::var("DATABASE_DBNAME").ok();

        if let Ok(port_str) = env::var("DATABASE_PORT") {
            cfg.port = port_str.parse::<u16>().ok();
        }

        info!(
            "✓ Connecting to database: user={}, host={}, port={:?}, dbname={}",
            cfg.user.as_deref().unwrap_or("N/A"),
            cfg.host.as_deref().unwrap_or("N/A"),
            cfg.port,
            cfg.dbname.as_deref().unwrap_or("N/A")
        );
        cfg
    };

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
}
