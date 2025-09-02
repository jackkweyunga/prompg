# PostgreSQL Prometheus Exporter

A high-performance, configurable Prometheus exporter for PostgreSQL, designed to run custom SQL queries and expose the results as metrics.

The entire monitoring stack (Exporter, Postgres, Prometheus, Grafana) is containerized for easy setup.

## Quick Start (Local Development)

This will launch the exporter and a full monitoring stack (Postgres, Prometheus, Grafana).

### 1. Configure Database Connection

Create a `.env` file in the project root. The exporter connects to PostgreSQL using one of two methods, in order of priority:

1.  **`DATABASE_URL` (Recommended):** A single connection string.
2.  **Individual `DATABASE_*` variables:** If `DATABASE_URL` is not set, the exporter will use these as a fallback.

**Example `.env`:**
```env
# .env

# Recommended: Use a single connection URL. If this is uncommented, it will be used.
# DATABASE_URL="postgres://postgres:password@postgres:5432/postgres"

# Fallback: Individual variables are used if DATABASE_URL is not set.
# For the local Docker setup, the host should be `postgres`.
DATABASE_USER=postgres
DATABASE_PASSWORD=password
DATABASE_HOST=postgres
DATABASE_PORT=5432
DATABASE_DBNAME=postgres
```

### 2. Define Your Metrics

Modify the `config/metrics.toml` file to add your custom SQL queries. The exporter will hot-reload this file if you make changes while it's running.

**Example `config/metrics.toml`:**
```toml
# A simple, single-value metric
[[queries]]
name = "pg_database_size_bytes"
help = "Size of the database in bytes"
query = "SELECT pg_database_size(current_database())::float8 as size_bytes"
value_column = "size_bytes"

# A metric with dynamic labels
[[queries]]
name = "users_by_status_total"
help = "Total number of users by status."
value_column = "value"
label_columns = ["status"] # Creates a label from the 'status' column
query = "SELECT status, COUNT(*)::float8 AS value FROM users GROUP BY status"
```

### 3. Run the Stack

Launch everything with a single command:

```bash
docker compose up -d
```

## Service Endpoints

Once running, the services are available at:

| Service           | URL                               | Credentials      |
| ----------------- | --------------------------------- | ---------------- |
| **Metrics Exporter** | `http://localhost:8080/metrics
