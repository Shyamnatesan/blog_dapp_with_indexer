use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub address: String,
    pub post_count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub user_address: String,
    pub title: String,
    pub content: String,
    pub likes: i64,
    pub post_index: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserStats {
    pub user_address: String,
    pub total_likes: i64,
    pub total_posts: i64,
    pub latest_post_index: Option<i64>,
}
