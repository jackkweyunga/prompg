# PostgreSQL Metrics Exporter for Prometheus

This is a high-performance, configurable Prometheus exporter for PostgreSQL. It is designed to run custom SQL queries against a database and expose the results as Prometheus metrics, which can then be visualized in Grafana.

The entire stack, including the exporter, Prometheus, and Grafana, is containerized and can be launched with a single command.

## Features

-   **Custom SQL Queries:** Define your own metrics by writing SQL queries in a simple TOML configuration file.
-   **Dynamic Labels:** Generate multiple time series from a single query by specifying which columns should be used as Prometheus labels.
-   **Hot-Reloading:** The exporter watches the metrics configuration file for changes and automatically reloads its queries without requiring a restart.
-   **Fully Containerized:** The entire monitoring stack (Exporter, Postgres, Prometheus, Grafana) is managed via Docker Compose for easy setup and deployment.
-   **Structured Logging:** Comprehensive logging provides clear visibility into the exporter's operations, with configurable verbosity.
-   **Secure by Default:** The exporter runs as a non-root user inside its container.

## Getting Started

### Prerequisites

-   [Docker](https://docs.docker.com/get-docker/)
-   [Docker Compose](https://docs.docker.com/compose/install/)

### 1. Configure the Database Connection

The exporter needs to know how to connect to your PostgreSQL database. This is configured using a `.env` file.

Create a file named `.env` in the root of this project and fill it with your database credentials. You can use the template below:

```env
# .env file for metrics-exporter

# PostgreSQL Database Connection Details
DATABASE_USER=postgres
DATABASE_PASSWORD=your_db_password
DATABASE_HOST=postgres # Use the service name from docker-compose, or a real IP
DATABASE_PORT=5432
DATABASE_DBNAME=postgres
```
*Note: In the default `docker-compose.yml` setup, `APP_DATABASE__HOST` should be `postgres` to connect to the included Postgres container.*

### 2. Configure Your Metrics

Metrics are defined in `config/metrics.toml`. You can add as many queries as you need. The file is hot-reloaded, so any changes you save will be picked up by the running exporter automatically.

Each query has the following structure:

-   `name`: The name of the Prometheus metric.
-   `help`: The help text for the metric.
-   `query`: The SQL query to execute.
-   `value_column`: The name of the column in the query result that contains the metric value.
-   `label_columns` (Optional): A list of columns that will be converted into Prometheus labels.

**Example `metrics.toml`:**

```toml
# A simple metric without labels
[[queries]]
name = "pg_database_size_bytes"
help = "Size of the database in bytes"
query = "SELECT pg_database_size(current_database())::float8 as size_bytes"
value_column = "size_bytes"

# A metric that generates multiple time series with labels
[[queries]]
name = "accounts_users_by_status_total"
help = "Total number of users by verification status."
value_column = "value"
label_columns = ["verification_status"]
query = """
  SELECT
    verification_status,
    COUNT(*)::float8 AS value
  FROM
    public.accounts_user
  GROUP BY
    verification_status
"""

### 3. Configure Logging (Optional)

The application uses structured logging to provide insight into its operations. The verbosity of the logs is controlled by the `RUST_LOG` environment variable, which can be set in your `.env` file.

The default log level is `info`.

-   **`RUST_LOG=info`**: (Default) Shows startup messages, successful configuration reloads, and connection info.
-   **`RUST_LOG=warn`**: Quieter. Only shows warnings for failed SQL queries or issues parsing results.
-   **`RUST_LOG=error`**: Only shows critical errors, such as a failure to connect to the database or an invalid configuration file.
-   **`RUST_LOG=debug`**: Very verbose. Logs every metric value that is set for every scrape. Useful for debugging a specific query.
```

### 4. Run the Entire Stack

With your configuration in place, you can launch the exporter, Prometheus, and Grafana with a single command:

```bash
docker compose up -d
```

This command will build the `metrics-exporter` image, pull the official images for the other services, and start everything in the background.

## Service Endpoints

Once the stack is running, you can access the different services in your browser:

-   **Metrics Exporter:** [http://localhost:8080/metrics](http://localhost:8080/metrics)
    -   (See the raw metrics text that Prometheus scrapes)
-   **Prometheus:** [http://localhost:9090](http://localhost:9090)
    -   (Explore metrics and check scrape status under `Status -> Targets`)
-   **Grafana:** [http://localhost:3000](http://localhost:3000)
    -   (Login with `admin` / `password`)

## Using a Custom `metrics.toml`

The `docker-compose.yml` file is configured to allow you to easily swap out the `metrics.toml` file without rebuilding the Docker image. This is achieved using a Docker volume.

The following line in `docker-compose.yml` mounts your local file into the container:

```yaml
volumes:
  # Mounts your local config into the container, making it replaceable
  - ./config/metrics.toml:/app/config/metrics.toml:ro
```

To use a different configuration file:

1.  Create a new TOML file somewhere on your host machine (e.g., `/path/to/my-production-metrics.toml`).
2.  Stop the stack: `docker compose down`.
3.  Modify the `volumes` section in `docker-compose.yml` to point to your new file:
    ```yaml
    volumes:
      - /path/to/my-production-metrics.toml:/app/config/metrics.toml:ro
    ```
4.  Start the stack again: `docker compose up -d`.

The exporter will now use your custom configuration file.
