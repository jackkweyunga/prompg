mod config;
mod db;
mod metrics;
mod queries;
mod state;

use crate::state::AppState;
use db::DbPool;
use log::{error, info, warn};
use notify::Watcher;
use notify_debouncer_full::new_debouncer;
use prometheus::{Encoder, TextEncoder};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

async fn metrics_handler(
    pool: DbPool,
    state: Arc<Mutex<AppState>>,
) -> Result<impl Reply, Rejection> {
    // Lock the state to get access to the metrics and registry.
    let state = state.lock().await;
    state.metrics.update(&pool).await;

    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    let mf = state.registry.gather();
    encoder.encode(&mf, &mut buffer).unwrap();

    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

#[tokio::main]
async fn main() {
    // Initialize the logger. This reads the RUST_LOG environment variable.
    env_logger::init();

    match dotenvy::dotenv() {
        Ok(path) => info!("✓ Loaded .env file from: {}", path.display()),
        Err(e) => warn!(
            "✗ Could not load .env file: {}. Relying on environment variables.",
            e
        ),
    }

    // Perform the initial configuration load.
    let config = config::Config::from_env().expect("FATAL: Failed to load initial configuration");
    let db_config = config.database.clone();

    let pool = db::create_pool(&db_config).expect("FATAL: Failed to create database pool");
    info!(
        "✓ Connected to database: user={}, host={}, port={}, dbname={}",
        db_config.user, db_config.host, db_config.port, db_config.dbname
    );

    // Create the initial, shared application state.
    let initial_state =
        AppState::new(&config).expect("FATAL: Failed to create initial metrics state");
    let shared_state = Arc::new(Mutex::new(initial_state));

    // Spawn the background task to watch for configuration changes.
    tokio::spawn(watch_config_changes(Arc::clone(&shared_state)));

    // Set up the warp filter chain.
    let with_pool = warp::any().map(move || pool.clone());
    let with_state = warp::any().map(move || Arc::clone(&shared_state));

    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(with_pool)
        .and(with_state)
        .and_then(metrics_handler);

    info!("✓ Starting server at http://127.0.0.1:8080/metrics");
    warp::serve(metrics_route).run(([127, 0, 0, 1], 8080)).await;
}

/// Watches for changes in `config/metrics.toml` and reloads the application state.
async fn watch_config_changes(shared_state: Arc<Mutex<AppState>>) {
    let (tx, rx) = std::sync::mpsc::channel();

    // Create a debouncer to handle file system events.
    let mut debouncer =
        new_debouncer(Duration::from_secs(2), None, tx).expect("Failed to create file watcher");

    debouncer
        .watcher()
        .watch(
            Path::new("config/metrics.toml"),
            notify::RecursiveMode::NonRecursive,
        )
        .expect("Failed to start watching config/metrics.toml");

    info!("✓ Watching config/metrics.toml for changes...");

    // This loop processes events from the file watcher.
    for res in rx {
        match res {
            Ok(events) => {
                if events
                    .iter()
                    .any(|e| e.kind.is_modify() || e.kind.is_create())
                {
                    info!("✓ Change detected in config/metrics.toml, attempting to reload...");

                    // Attempt to reload the configuration.
                    match config::Config::from_env() {
                        Ok(new_config) => {
                            // Create a new AppState based on the new config.
                            match AppState::new(&new_config) {
                                Ok(new_state) => {
                                    // Lock the shared state and replace it with the new state.
                                    let mut state_guard = shared_state.lock().await;
                                    *state_guard = new_state;
                                    info!("✓ Successfully reloaded metrics configuration.");
                                }
                                Err(e) => {
                                    error!(
                                        "✗ Error creating new metrics from reloaded config: {}",
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("✗ Error reloading config/metrics.toml: {}. Keeping previous configuration.", e);
                        }
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("✗ File watcher error: {:?}", error);
                }
            }
        }
    }
}
