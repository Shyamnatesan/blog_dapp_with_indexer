use anchor_lang::prelude::{borsh::BorshDeserialize, *};

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

pub struct Event {
    pub discriminator: [u8; 8],
    pub data: Vec<u8>,
}
