use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Gauge,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryConfig {
    pub name: String,
    pub help: String,
    pub query: String,
    pub value_column: String,
    #[serde(default)]
    pub label_columns: Vec<String>,
    pub metric_type: MetricType,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub queries: Vec<QueryConfig>,
}

impl Config {
    /// Loads query configuration from `config/metrics.toml`.
    /// This file is optional.
    pub fn from_file() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config/metrics").required(false))
            .build()?
            .try_deserialize()
    }
}
