-- Migration: Add statistics tables for precalculated metrics
-- This enables O(1) retrieval of user and deck statistics

-- User-level statistics
CREATE TABLE IF NOT EXISTS user_stats (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    correct_reviews INTEGER NOT NULL DEFAULT 0,
    days_studied INTEGER NOT NULL DEFAULT 0,
    last_active_date DATE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Deck-level statistics
CREATE TABLE IF NOT EXISTS deck_stats (
    deck_id UUID PRIMARY KEY REFERENCES decks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total_cards INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    correct_reviews INTEGER NOT NULL DEFAULT 0,
    days_studied INTEGER NOT NULL DEFAULT 0,
    last_active_date DATE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_stats_user_id ON user_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_deck_stats_deck_id ON deck_stats(deck_id);
CREATE INDEX IF NOT EXISTS idx_deck_stats_user_id ON deck_stats(user_id);

-- Initialize user_stats for existing users
INSERT INTO user_stats (user_id, total_reviews, correct_reviews, days_studied)
SELECT 
    u.id,
    0,
    0,
    0
FROM users u
WHERE NOT EXISTS (SELECT 1 FROM user_stats WHERE user_id = u.id);

-- Initialize deck_stats for existing decks
INSERT INTO deck_stats (deck_id, user_id, total_cards, total_reviews, correct_reviews, days_studied)
SELECT 
    d.id,
    d.user_id,
    COALESCE((SELECT COUNT(*) FROM cards WHERE deck_id = d.id), 0),
    0,
    0,
    0
FROM decks d
WHERE NOT EXISTS (SELECT 1 FROM deck_stats WHERE deck_id = d.id);
