use crate::config::Config;
use crate::metrics::Metrics;
use prometheus::Registry;

/// A struct to hold the application's state that needs to be shared and reloaded.
/// This includes the Prometheus registry and the custom metrics definitions.
pub struct AppState {
    pub registry: Registry,
    pub metrics: Metrics,
}

impl AppState {
    /// Creates a new `AppState` from the given configuration.
    ///
    /// This function initializes the Prometheus registry, creates all the metrics
    /// based on the query configurations, and registers them.
    pub fn new(config: &Config) -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        let metrics = Metrics::new(&config.queries);
        metrics.register(&registry)?;
        Ok(Self { registry, metrics })
    }
}
