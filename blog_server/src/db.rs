use sqlx::{SqlitePool, pool::PoolOptions, sqlite::SqliteConnectOptions};

pub async fn init_db_pool(number_of_connections: u32) -> SqlitePool {
    let option = SqliteConnectOptions::new()
        .filename("blog.db")
        .create_if_missing(true);

    let pool = PoolOptions::new()
        .max_connections(number_of_connections)
        .connect_with(option)
        .await
        .expect("Failed to create connection pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Database connected and migrations applied");

    pool
}
