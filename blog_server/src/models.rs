use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub pubkey: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub likes: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserStats {
    pub user_address: String,
    pub total_likes: i64,
    pub total_posts: i64,
    pub latest_post_index: Option<i64>,
}
