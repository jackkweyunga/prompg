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

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub queries: Vec<QueryConfig>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let result = config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .add_source(config::File::with_name("config/metrics"))
            .build()?
            .try_deserialize();

        if let Ok(config) = &result {
            dbg!(config);
        }
        result
    }
}
