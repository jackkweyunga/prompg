use crate::config::QueryConfig;
use crate::db::DbPool;
use log::{debug, error, warn};
use prometheus::{Gauge, GaugeVec, Opts, Registry};

/// An enum to hold either a single Gauge or a GaugeVec for labeled metrics.
/// This allows us to handle both labeled and unlabeled metrics in a unified way.
#[derive(Clone)]
enum Metric {
    Gauge(Gauge),
    GaugeVec(GaugeVec),
}

#[derive(Clone)]
pub struct Metrics {
    // Each metric is now a tuple of its config and the `Metric` enum.
    metrics: Vec<(QueryConfig, Metric)>,
}

impl Metrics {
    pub fn new(query_configs: &[QueryConfig]) -> Self {
        let metrics = query_configs
            .iter()
            .map(|qc| {
                let opts = Opts::new(qc.name.clone(), qc.help.clone());
                let metric = if qc.label_columns.is_empty() {
                    // No labels, create a simple Gauge.
                    Metric::Gauge(Gauge::with_opts(opts).unwrap())
                } else {
                    // Labels are present, create a GaugeVec.
                    let label_names: Vec<&str> =
                        qc.label_columns.iter().map(AsRef::as_ref).collect();
                    Metric::GaugeVec(GaugeVec::new(opts, &label_names).unwrap())
                };
                (qc.clone(), metric)
            })
            .collect();

        Self { metrics }
    }

    pub fn register(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        for (_, metric) in &self.metrics {
            match metric {
                Metric::Gauge(g) => registry.register(Box::new(g.clone()))?,
                Metric::GaugeVec(gv) => registry.register(Box::new(gv.clone()))?,
            }
        }
        Ok(())
    }

    pub async fn update(&self, pool: &DbPool) {
        let client = match pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get database client from pool: {}", e);
                return;
            }
        };

        for (qc, metric) in &self.metrics {
            match metric {
                Metric::Gauge(gauge) => {
                    // Handle single-value, unlabeled metrics.
                    let row = match client.query_one(qc.query.as_str(), &[]).await {
                        Ok(row) => row,
                        Err(e) => {
                            warn!("Failed to execute query for metric '{}': {}", qc.name, e);
                            continue;
                        }
                    };

                    let value: f64 = match row.try_get(qc.value_column.as_str()) {
                        Ok(val) => val,
                        Err(e) => {
                            warn!(
                                "Failed to get value from column '{}' for metric '{}': {}",
                                qc.value_column, qc.name, e
                            );
                            continue;
                        }
                    };
                    gauge.set(value);
                    debug!("Set metric '{}' to {}", qc.name, value);
                }
                Metric::GaugeVec(gauge_vec) => {
                    // Handle multi-value, labeled metrics.
                    gauge_vec.reset(); // Clear old values before setting new ones.
                    let rows = match client.query(qc.query.as_str(), &[]).await {
                        Ok(rows) => rows,
                        Err(e) => {
                            warn!("Failed to execute query for metric '{}': {}", qc.name, e);
                            continue;
                        }
                    };

                    for row in rows {
                        // Extract label values from the row.
                        let label_values: Vec<String> = qc
                            .label_columns
                            .iter()
                            .map(|label_name| row.get(label_name.as_str()))
                            .collect();

                        // Convert to &str for the prometheus crate.
                        let label_values_str: Vec<&str> =
                            label_values.iter().map(AsRef::as_ref).collect();

                        // Extract the metric value.
                        let value: f64 = match row.try_get(qc.value_column.as_str()) {
                            Ok(val) => val,
                            Err(e) => {
                                warn!(
                                    "Failed to get value from column '{}' for metric '{}' with labels {:?}: {}",
                                    qc.value_column, qc.name, label_values_str, e
                                );
                                continue; // Skip this row
                            }
                        };

                        gauge_vec.with_label_values(&label_values_str).set(value);
                        debug!(
                            "Set metric '{}' with labels {:?} to {}",
                            qc.name, label_values_str, value
                        );
                    }
                }
            }
        }
    }
}
