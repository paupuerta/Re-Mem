-- Database initialization script for Docker
-- This script creates the initial database schema

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create cards (flashcards) table
CREATE TABLE IF NOT EXISTS cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_cards_user_id ON cards(user_id);

-- Create reviews table
CREATE TABLE IF NOT EXISTS reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    grade INT NOT NULL CHECK (grade >= 0 AND grade <= 5),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_reviews_card_id ON reviews(card_id);
CREATE INDEX IF NOT EXISTS idx_reviews_user_id ON reviews(user_id);

-- Optional: Create FSRS state tracking table (future enhancement)
-- CREATE TABLE IF NOT EXISTS card_fsrs_states (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     card_id UUID NOT NULL UNIQUE REFERENCES cards(id),
--     stability FLOAT8 NOT NULL DEFAULT 1.0,
--     difficulty FLOAT8 NOT NULL DEFAULT 5.0,
--     last_review TIMESTAMP WITH TIME ZONE,
--     next_review TIMESTAMP WITH TIME ZONE,
--     reps INT NOT NULL DEFAULT 0,
--     lapses INT NOT NULL DEFAULT 0,
--     state VARCHAR(50),
--     updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
-- );
