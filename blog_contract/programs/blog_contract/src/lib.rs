use anchor_lang::prelude::*;

declare_id!("TUfhbucqRBwNNS6Ai1DXEZeJ4LwErVKACmEegz3JmUT");

#[program]
pub mod blog_contract {

    use super::*;

    pub fn create_user_blog_account(ctx: Context<CreateUserBlogAccount>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.post_count = 0;
        msg!("User Blog Account created successfully");
        Ok(())
    }

    pub fn create_post(ctx: Context<CreatePost>, title: String, content: String) -> Result<()> {
        let post = &mut ctx.accounts.post;
        let user = &mut ctx.accounts.user;

        let index = user.post_count;
        post.post_index = index;
        post.title = title;
        post.content = content;
        post.likes = 0;
        post.author = ctx.accounts.payer.key();
        post.created_at = Clock::get()?.unix_timestamp;
        post.bump = ctx.bumps.post;

        user.post_count = user
            .post_count
            .checked_add(1)
            .ok_or_else(|| error!(ErrorCode::PostCountOverflow))?;

        msg!(
            "Post created successfully: {} by {}",
            post.title,
            post.author
        );

        msg!(
            "{} has created {} posts",
            ctx.accounts.payer.key(),
            user.post_count
        );
        Ok(())
    }

    pub fn like_post(ctx: Context<LikePost>, author: Pubkey, post_index: u64) -> Result<()> {
        let post = &mut ctx.accounts.post;
        post.likes = post
            .likes
            .checked_add(1)
            .ok_or_else(|| error!(ErrorCode::PostCountOverflow))?;
        msg!(
            "Post liked successfully: {} now has {} likes",
            post.title,
            post.likes
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateUserBlogAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + User::INIT_SPACE,
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePost<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    #[account(
        init,
        payer = payer,
        space = 8 + Post::INIT_SPACE,
        seeds = [b"post", payer.key().as_ref(), &user.post_count.to_le_bytes()],
        bump
    )]
    pub post: Account<'info, Post>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(author: Pubkey, post_index: u64)]
pub struct LikePost<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"post", author.as_ref(), &post_index.to_le_bytes()],
        bump,
    )]
    pub post: Account<'info, Post>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Post {
    pub post_index: u64,
    #[max_len(100)]
    pub title: String,
    #[max_len(1000)]
    pub content: String,
    pub likes: u64,
    pub author: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct User {
    pub post_count: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Post count overflow")]
    PostCountOverflow,
}
