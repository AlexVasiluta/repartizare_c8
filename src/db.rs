use sqlx::{
    migrate,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions,
};
use std::str::FromStr;

pub async fn create_pool(dsn: &str, create: bool) -> Result<sqlx::SqlitePool, sqlx::Error> {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::from_str(dsn)?
                .foreign_keys(true)
                .create_if_missing(create)
                .disable_statement_logging()
                .clone(),
        )
        .await?;

    migrate!().run(&db).await?;

    Ok(db)
}
