-- Migration to add deck support to existing database
-- Run this if you have an existing database that needs updating

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create decks table
CREATE TABLE IF NOT EXISTS decks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_decks_user_id ON decks(user_id);

-- Add deck_id column to cards table (if not exists)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='cards' AND column_name='deck_id') THEN
        ALTER TABLE cards ADD COLUMN deck_id UUID REFERENCES decks(id) ON DELETE SET NULL;
        CREATE INDEX idx_cards_deck_id ON cards(deck_id);
    END IF;
END $$;

-- Add answer_embedding column to cards table (if not exists)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='cards' AND column_name='answer_embedding') THEN
        ALTER TABLE cards ADD COLUMN answer_embedding vector(1536);
    END IF;
END $$;
