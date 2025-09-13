use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    AppState,
    models::{Post, UserStats},
};

pub async fn fetch_all_posts_by_user(
    Path(author): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let pool = state.db_pool.as_ref();

    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_address = ?1 ORDER BY created_at DESC",
    )
    .bind(&author)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(posts))
}

pub async fn fetch_post_by_user(
    Path(author): Path<String>,
    Path(post_index): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let pool = state.db_pool.as_ref();

    let post = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_address = ?1 AND post_index = ?2",
    )
    .bind(&author)
    .bind(post_index as i64)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if matches!(e, sqlx::Error::RowNotFound) {
            (StatusCode::NOT_FOUND, "Post not found".to_string())
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        }
    })?;

    Ok(Json(post))
}
pub async fn fetch_top_k_most_liked_posts(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let k = params.get("k").and_then(|v| v.parse().ok()).unwrap_or(10); // Default to 10 if not provided
    let pool = state.db_pool.as_ref();

    let posts = sqlx::query_as::<_, Post>(
        "
    SELECT * FROM posts ORDER BY likes DESC LIMIT ?1",
    )
    .bind(&k)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(posts))
}

pub async fn fetch_user_stats(
    Path(user): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<UserStats>, (StatusCode, String)> {
    let pool = state.db_pool.as_ref();

    // Fetch total likes, total posts, latest post index from `posts` table
    let stats = sqlx::query_as::<_, UserStats>(
        "SELECT user_address, 
            COUNT(*) as total_posts, 
            COALESCE(SUM(likes), 0) as total_likes, 
            MAX(post_index) as latest_post_index
     FROM posts
     WHERE user_address = ?1
     GROUP BY user_address",
    )
    .bind(&user)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(stats))
}

pub async fn fetch_trending_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let pool = state.db_pool.as_ref();

    // Fetch posts with likes greater than 100, ordered by likes in descending order
    let posts =
        sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE likes > 100 ORDER BY likes DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    Ok(Json(posts))
}
