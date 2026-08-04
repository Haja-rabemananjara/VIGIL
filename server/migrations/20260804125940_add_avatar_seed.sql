-- Add migration script here
ALTER TABLE users ADD COLUMN avatar_seed TEXT DEFAULT NULL;