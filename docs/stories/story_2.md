# Historia 2: Deck and Card Creation - User Guide

## Overview
Historia 2 adds the ability to organize flashcards into decks and create new cards with AI-powered semantic embeddings for future similarity search features.

## New Features

### Backend Features

#### 1. Deck Management
- **Create Deck**: POST `/users/{user_id}/decks`
  ```json
  {
    "name": "Spanish Vocabulary",
    "description": "Basic Spanish words and phrases"
  }
  ```

- **List User Decks**: GET `/users/{user_id}/decks`
  - Returns all decks for a specific user

- **Get Cards by Deck**: GET `/decks/{deck_id}/cards`
  - Returns all cards within a specific deck

#### 2. Enhanced Card Creation
- **Create Card with Deck**: POST `/users/{user_id}/cards`
  ```json
  {
    "deck_id": "uuid-of-deck",  // Optional
    "question": "What is 'hello' in Spanish?",
    "answer": "hola"
  }
  ```
  - Cards can optionally belong to a deck
  - AI embeddings are automatically generated for the answer
  - Embeddings stored as vector(1536) for future semantic search

#### 3. Database Schema
- **New `decks` table**: Organizes cards into collections
- **Extended `cards` table**:
  - `deck_id`: Optional foreign key to decks
  - `answer_embedding`: Vector embeddings for semantic search
- **pgvector extension**: Enables similarity search (future feature)

### Frontend Features

#### 1. Deck Management Screen
Navigate to deck management from the home screen via:
- AppBar icon button (top right)
- "Manage Decks" button on home screen

Features:
- **List all decks** with pull-to-refresh
- **Create new deck** via floating action button
- **View deck cards** by tapping on any deck
- Empty state with helpful message when no decks exist

#### 2. Create Deck Dialog
Accessible via "New Deck" button on DecksScreen:
- **Name field**: Required, minimum 2 characters
- **Description field**: Optional, supports multiple lines
- Form validation with error messages
- Loading indicator during creation
- Success/error feedback via snackbar

#### 3. Deck Cards Screen
Shows all cards within a specific deck:
- **Card list** with question and answer preview
- **Create card** via floating action button
- Cards automatically assigned to current deck
- Pull-to-refresh to reload cards
- Empty state when deck has no cards
- Tap card to start review

#### 4. Create Card Dialog
Accessible from DeckCardsScreen:
- **Question field**: Required, minimum 3 characters
- **Answer field**: Required, minimum 1 character
- **Auto deck assignment**: Cards created from deck screen are automatically assigned
- Form validation
- Success/error feedback
- Info banner showing deck assignment

## Technical Architecture

### Backend (Rust)

**Domain Layer**:
- `Deck` entity: Collection organizer
- `Card` entity: Extended with `deck_id` and `answer_embedding`
- `DeckRepository` trait: CRUD operations
- `CardRepository` trait: Extended with `find_by_deck`
- `EmbeddingService` trait: Generate semantic embeddings

**Application Layer**:
- `CreateDeckUseCase`: Business logic for deck creation
- `CreateCardUseCase`: Card creation with automatic embedding generation
- `GetDecksUseCase`: Retrieve user's decks
- DTOs for request/response serialization

**Infrastructure Layer**:
- `PgDeckRepository`: PostgreSQL implementation
- `PgCardRepository`: Updated with deck and embedding support
- `OpenAIValidator`: Implements `EmbeddingService` using text-embedding-3-small
- pgvector integration for efficient vector storage

### Frontend (Flutter)

**Domain Layer**:
- `Deck` entity with JSON serialization
- `Card` entity with optional `deckId`
- `DeckRepository` interface
- Use cases: `CreateDeck`, `GetDecks`

**Data Layer**:
- `DeckRepositoryImpl`: API integration
- `CardRepositoryImpl`: Updated to handle deck assignments

**Presentation Layer**:
- `DecksScreen`: List and manage decks
- `DeckCardsScreen`: View cards in deck
- `CreateDeckDialog`: Deck creation form
- `CreateCardDialog`: Card creation form
- Riverpod providers for state management

## Error Handling

### Backend
- Graceful degradation: If embedding generation fails, card is still created
- Proper error responses with status codes
- Database constraints ensure data integrity

### Frontend
- Form validation with user-friendly error messages
- Network error handling with retry options
- Loading states during async operations
- Success/error feedback via snackbars

## Testing

### Backend Tests
- 17 unit tests passing
- CreateDeckUseCase tests (creation with/without description)
- CreateCardUseCase tests (with embedding, without deck, embedding failure)
- All domain and infrastructure tests

### Frontend
- Form validation tests
- Use case tests with mock repositories
- Widget tests for dialogs and screens

## Future Enhancements

### Planned Features
1. **Semantic Search**: Use embeddings to find similar cards
2. **Deck Statistics**: Track learning progress per deck
3. **Deck Sharing**: Share decks with other users
4. **Import/Export**: Bulk import cards from CSV/JSON
5. **Deck Templates**: Pre-made decks for common topics

### Performance Optimizations
1. **Batch Embedding Generation**: Generate multiple embeddings in parallel
2. **Caching**: Cache deck lists and card counts
3. **Pagination**: Add pagination for large deck/card lists
4. **Vector Indexing**: Optimize similarity search with HNSW index

## Migration Guide

If you have an existing ReMem installation:

1. **Update Docker Image**:
   ```yaml
   # docker-compose.yml
   postgres:
     image: pgvector/pgvector:pg15  # Changed from postgres:15-alpine
   ```

2. **Run Migration**:
   ```bash
   docker exec -i re_mem_postgres psql -U re_mem -d re_mem < scripts/migrate_add_decks.sql
   ```

3. **Restart Backend**:
   ```bash
   docker-compose restart backend
   ```

For new installations, everything is included in `scripts/init.sql`.

## API Examples

### Create a Deck
```bash
curl -X POST http://localhost:3000/users/{user_id}/decks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "French Verbs",
    "description": "Common French irregular verbs"
  }'
```

### Create a Card with Deck Assignment
```bash
curl -X POST http://localhost:3000/users/{user_id}/cards \
  -H "Content-Type: application/json" \
  -d '{
    "deck_id": "{deck_id}",
    "question": "How do you say 'to be' in French?",
    "answer": "être"
  }'
```

### List All Decks
```bash
curl http://localhost:3000/users/{user_id}/decks
```

### Get Cards in Deck
```bash
curl http://localhost:3000/decks/{deck_id}/cards
```

## Troubleshooting

### Backend Issues

**Error: "column deck_id does not exist"**
- Run the migration script: `scripts/migrate_add_decks.sql`
- Or recreate the database with `docker-compose down -v && docker-compose up -d`

**Error: "extension vector does not exist"**
- Ensure using `pgvector/pgvector:pg15` image
- Restart PostgreSQL container

**Embeddings not generated**
- Check OPENAI_API_KEY environment variable
- System uses FallbackValidator if no API key (cards still created)

### Frontend Issues

**Decks not loading**
- Check backend is running on port 3000
- Verify API_BASE_URL in frontend configuration
- Check network connectivity

**Forms not submitting**
- Check form validation errors
- Verify all required fields are filled
- Check browser console for JavaScript errors

## Performance Considerations

### Backend
- Embedding generation adds ~200-500ms to card creation
- pgvector storage is efficient (1536 floats = ~6KB per card)
- Consider rate limiting for OpenAI API calls

### Frontend
- Deck list refreshes on navigation back
- Card list filtered client-side (consider server-side for large decks)
- Images/avatars not yet implemented (future enhancement)

## Security Considerations

- User ID validation required (not yet implemented)
- Deck ownership verification on all operations
- SQL injection prevented via parameterized queries
- XSS protection via React/Flutter framework defaults
- CORS configured for development (restrict in production)

---

**Implementation Complete**: Historia 2 successfully adds deck organization and AI-powered card creation with semantic embeddings.
