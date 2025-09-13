-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL UNIQUE,
    post_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_users_address ON users(address);

-- Create posts table
CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_address TEXT NOT NULL REFERENCES users(address) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    likes INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    post_index INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_posts_user_address ON posts(user_address);
