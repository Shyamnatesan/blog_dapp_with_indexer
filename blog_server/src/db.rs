use std::{env, fs};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub async fn init_db_pool(number_of_connections: u32) -> SqlitePool {
    let option = SqliteConnectOptions::new()
        .filename("blog.db")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(option).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Database connected and migrations applied");

    pool
}
