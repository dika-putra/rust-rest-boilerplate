use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Ok};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use tracing::info;

pub type ConnectionPool = Pool<Sqlite>;

/// Embedded SQLite database
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: ConnectionPool,
}

impl Database {
    pub async fn connect(connection_string: &str, run_migrations: bool) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(connection_string)
            .context("DATABASE_URL is not a valid SQLite url")?
            .create_if_missing(true)
            .foreign_keys(true);

        // SQLite creates the file but not its folder. sqlx 0.6 has no getter for the parsed
        // filename, so take it from the url: sqlite://data/app.db?mode=rwc -> data/app.db
        let file = connection_string
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:")
            .split('?')
            .next()
            .unwrap_or_default();
        if let Some(dir) = Path::new(file).parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir).context("could not create the database folder")?;
            }
        }

        // One connection: with WAL and a pool of 5, a session inserted on one connection was
        // not visible to the next query on another (sqlx 0.6.3), and `:memory:` gives every
        // connection its own empty database. Writes are serialized, which is fine at this scale;
        // revisit if request volume grows.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .context("error while initializing the database connection pool")?;

        if run_migrations {
            info!("migrations enabled, running...");
            sqlx::migrate!()
                .run(&pool)
                .await
                .context("error while running database migrations")?;
            info!("migrations successfully ran, initializing axum server...");
        }

        Ok(Self { pool })
    }
}
