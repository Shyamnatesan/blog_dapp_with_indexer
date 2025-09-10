use anchor_lang::prelude::borsh::BorshDeserialize;
use anchor_lang::prelude::*;
use base64::{Engine, engine::general_purpose};

use sha2::{Digest, Sha256};
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_commitment_config::CommitmentConfig;

#[derive(Debug, BorshDeserialize)]
pub struct UserCreated {
    pub user: Pubkey,
    pub post_count: u64,
}

#[derive(Debug, BorshDeserialize)]
pub struct PostCreated {
    pub author: Pubkey,
    pub post_index: u64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, BorshDeserialize)]
pub struct PostLiked {
    pub liker: Pubkey,
    pub author: Pubkey,
    pub post_index: u64,
    pub total_likes: u64,
}

fn event_discriminator(name: &str) -> [u8; 8] {
    // Anchor's event discriminator = sha256("event:<EventName>")[..8]
    let mut hasher = Sha256::new();
    hasher.update(format!("event:{}", name).as_bytes());
    let hash = hasher.finalize();
    hash[..8].try_into().expect("slice with incorrect length")
}

#[tokio::main]
async fn main() {
    // Program ID
    let program_id = Pubkey::from_str_const("TUfhbucqRBwNNS6Ai1DXEZeJ4LwErVKACmEegz3JmUT");

    // precompute discriminators
    let user_disc = event_discriminator("UserCreated");
    let post_disc = event_discriminator("PostCreated");
    let liked_disc = event_discriminator("PostLiked");

    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };

    let filter = RpcTransactionLogsFilter::Mentions(vec![program_id.to_string()]);

    // Connect to localnet WebSocket
    let ws_url = "ws://localhost:8900"; // your localnet ws port
    let (mut client, receiver) = PubsubClient::logs_subscribe(ws_url, filter, config).unwrap();

    println!("Listening for events from program: {}", program_id);

    while let Ok(log_info) = receiver.recv() {
        for log in log_info.value.logs.iter() {
            if let Some(stripped) = log.strip_prefix("Program data: ") {
                match general_purpose::STANDARD.decode(stripped) {
                    Ok(bytes) if bytes.len() >= 8 => {
                        let (disc, data) = bytes.split_at(8);
                        if disc == user_disc.as_ref() {
                            match UserCreated::try_from_slice(data) {
                                Ok(ev) => {
                                    let user_pk = Pubkey::new_from_array(ev.user.to_bytes());
                                    println!(
                                        "UserCreated: user={} post_count={}",
                                        user_pk, ev.post_count
                                    );
                                }
                                Err(e) => eprintln!("UserCreated decode error: {:?}", e),
                            }
                        } else if disc == post_disc.as_ref() {
                            match PostCreated::try_from_slice(data) {
                                Ok(ev) => {
                                    let author_pk = Pubkey::new_from_array(ev.author.to_bytes());

                                    println!(
                                        "PostCreated: author={} index={} title={} content={}",
                                        author_pk, ev.post_index, ev.title, ev.content
                                    );
                                }
                                Err(e) => eprintln!("PostCreated decode error: {:?}", e),
                            }
                        } else if disc == liked_disc.as_ref() {
                            match PostLiked::try_from_slice(data) {
                                Ok(ev) => {
                                    let liker_pk = Pubkey::new_from_array(ev.liker.to_bytes());
                                    let author_pk = Pubkey::new_from_array(ev.author.to_bytes());
                                    println!(
                                        "PostLiked: liker={} author={} index={} total_likes={}",
                                        liker_pk, author_pk, ev.post_index, ev.total_likes
                                    );
                                }
                                Err(e) => eprintln!("PostLiked decode error: {:?}", e),
                            }
                        } else {
                            // Unknown event discriminator — you can log or ignore
                            eprintln!("Unknown event discriminator: {:?}", disc);
                        }
                    }

                    Ok(_) => {
                        // valid base64 but too short
                        eprintln!("Invalid event data: too short");
                    }
                    Err(_) => {
                        // too-short payload
                        eprintln!("Program data too short");
                    }
                }
            }
        }
    }
}
