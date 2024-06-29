-- Add up migration script here
CREATE TABLE users (
    discord_id INTEGER PRIMARY KEY,
    ffxiv_username TEXT NOT NULL UNIQUE
);
