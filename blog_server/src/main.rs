mod db;
mod events;
mod indexer;
mod models;
mod service;

use std::sync::Arc;

use axum::{Router, routing::get};
use dotenv::dotenv;
use indexer::start_indexer;

use sqlx::SqlitePool;
use tokio::net::TcpListener;

use crate::service::{
    fetch_all_posts_by_user, fetch_post_by_user, fetch_top_k_most_liked_posts,
    fetch_trending_posts, fetch_user_stats,
};

#[derive(Clone)]
struct AppState {
    db_pool: Arc<SqlitePool>,
}

#[derive(Debug)]
pub enum AppError {
    DbError(sqlx::Error),
    NotFound,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::init_db_pool(5).await;

    let state = AppState {
        db_pool: Arc::new(pool),
    };

    tokio::spawn(async move {
        start_indexer().await;
    });

    let app = Router::new()
        .route("/posts/{author}", get(fetch_all_posts_by_user))
        .route("/post/{author}/{post_index}", get(fetch_post_by_user))
        .route("/top-posts", get(fetch_top_k_most_liked_posts))
        .route("/user-stats/{user}", get(fetch_user_stats))
        .route("/trending", get(fetch_trending_posts))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on http://127.0.0.1:8080");

    axum::serve(listener, app).await.unwrap();
}
