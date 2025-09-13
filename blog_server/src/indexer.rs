use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use anchor_lang::prelude::*;
use base64::{Engine, engine::general_purpose};
use sha2::{Digest, Sha256};
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use sqlx::{Result, SqlitePool};
use std::str::FromStr;
use tokio::{sync::mpsc, task};

use crate::{
    db,
    events::{Event, PostCreated, PostLiked, UserCreated},
};

pub async fn start_indexer() {
    let program_id_env = env::var("PROGRAM_ID").expect("PROGRAM_ID not set");
    let program_id = Pubkey::from_str(&program_id_env).unwrap();
    let pool = db::init_db_pool(5).await;
    println!("Starting indexer for program: {}", program_id);
    let user_disc = event_discriminator("UserCreated");
    let post_disc = event_discriminator("PostCreated");
    let liked_disc = event_discriminator("PostLiked");
    let (tx, mut rc) = mpsc::channel(1000);

    listen_for_blog_events_from_solana(program_id, tx).await;

    while let Some(event) = rc.recv().await {
        if event.discriminator == user_disc.as_ref() {
            match UserCreated::try_from_slice(&event.data) {
                Ok(ev) => create_new_user(&pool, ev).await.unwrap(),
                Err(e) => eprintln!("UserCreated decode error: {:?}", e),
            }
        } else if event.discriminator == post_disc.as_ref() {
            match PostCreated::try_from_slice(&event.data) {
                Ok(ev) => create_new_post(&pool, ev).await.unwrap(),
                Err(e) => eprintln!("PostCreated decode error: {:?}", e),
            }
        } else if event.discriminator == liked_disc.as_ref() {
            match PostLiked::try_from_slice(&event.data) {
                Ok(ev) => like_post(&pool, ev).await.unwrap(),
                Err(e) => eprintln!("PostLiked decode error: {:?}", e),
            }
        } else {
            // Unknown event discriminator — you can log or ignore
            eprintln!("Unknown event discriminator: {:?}", event.discriminator);
        }
    }
}

async fn listen_for_blog_events_from_solana(program_id: Pubkey, tx: mpsc::Sender<Event>) {
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };
    let filter = RpcTransactionLogsFilter::Mentions(vec![program_id.to_string()]);
    let ws_url = "ws://localhost:8900";
    let (client, receiver) =
        PubsubClient::logs_subscribe(ws_url, filter, config).expect("failed to subscribe");

    println!("Listening for events from program: {}", program_id);

    // Move receiver into blocking thread
    task::spawn_blocking(move || {
        let _client = client;
        while let Ok(log_info) = receiver.recv() {
            for log in log_info.value.logs.iter() {
                if let Some(stripped) = log.strip_prefix("Program data: ") {
                    match general_purpose::STANDARD.decode(stripped) {
                        Ok(bytes) if bytes.len() >= 8 => {
                            let (disc, data) = bytes.split_at(8);
                            let event = Event {
                                discriminator: disc.try_into().unwrap(),
                                data: data.to_vec(),
                            };
                            // blocking_send is intended for blocking contexts
                            if let Err(e) = tx.blocking_send(event) {
                                eprintln!("Failed to blocking_send event to indexer: {:?}", e);
                                return; // exit thread if channel closed
                            }
                        }
                        Ok(_) => eprintln!("Invalid event data: too short"),
                        Err(_) => eprintln!("Program data decode error"),
                    }
                }
            }
        }
        println!("Pubsub receiver loop ended");
    });
}

async fn create_new_user(pool: &SqlitePool, new_user: UserCreated) -> Result<(), sqlx::Error> {
    let addr = new_user.user.to_string();
    let post_count = new_user.post_count as i64;
    sqlx::query("INSERT INTO users (address, post_count) VALUES (?1, ?2)")
        .bind(&addr)
        .bind(&post_count)
        .execute(pool)
        .await?;

    println!("New user created: {} with post count {}", addr, post_count);
    Ok(())
}

async fn create_new_post(pool: &SqlitePool, new_post: PostCreated) -> Result<(), sqlx::Error> {
    let author = new_post.author.to_string();
    let post_index = new_post.post_index as i64;
    let title = new_post.title;
    let content = new_post.content;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "INSERT INTO posts (user_address, title, content, created_at, post_index) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&author)
    .bind(&title)
    .bind(&content)
    .bind(&now)
    .bind(&post_index)
    .execute(pool)
    .await?;

    println!(
        "New post created: author={} index={} title={} content={}",
        author, post_index, title, content
    );
    Ok(())
}

async fn like_post(pool: &SqlitePool, post_liked: PostLiked) -> Result<(), sqlx::Error> {
    let author = post_liked.author.to_string();
    let post_index = post_liked.post_index as i64;
    let likes = post_liked.total_likes as i64;

    sqlx::query("UPDATE POSTS SET likes = ?1 WHERE user_address = ?2 AND post_index = ?3")
        .bind(&likes)
        .bind(&author)
        .bind(&post_index)
        .execute(pool)
        .await?;

    println!(
        "Post liked: author={} index={} total_likes={}",
        author, post_index, post_liked.total_likes
    );

    Ok(())
}

fn event_discriminator(name: &str) -> [u8; 8] {
    // Anchor's event discriminator = sha256("event:<EventName>")[..8]
    let mut hasher = Sha256::new();
    hasher.update(format!("event:{}", name).as_bytes());
    let hash = hasher.finalize();
    hash[..8].try_into().expect("slice with incorrect length")
}
