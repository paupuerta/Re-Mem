-- Migration: Add authentication fields to users table
-- Run this against existing databases that were initialized without password_hash

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);
