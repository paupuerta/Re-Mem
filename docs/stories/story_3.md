# Story 3: Delete Decks and Cards

## Overview

Adds delete functionality for decks and cards, allowing users to remove unwanted content. When a deck is deleted, cards within it are preserved but moved to "No Deck" (deck_id set to NULL via ON DELETE SET NULL constraint).

## Backend Implementation

### Domain Layer

No changes needed - repositories already had delete methods defined.

### Use Cases Created

1. **DeleteDeckUseCase** (`src/application/use_cases/delete_deck.rs`)
   - Validates deck exists
   - Verifies user owns the deck (authorization check)
   - Calls repository delete
   - Returns `AuthorizationError` if user doesn't own deck
   - Returns `NotFound` if deck doesn't exist

2. **DeleteCardUseCase** (`src/application/use_cases/delete_card.rs`)
   - Validates card exists
   - Verifies user owns the card
   - Calls repository delete
   - Same error handling pattern as DeleteDeck

### Services

Extended `CardService` and `DeckService` with delete methods:

- `delete_card(card_id, user_id)` - deletes card with authorization
- `delete_deck(deck_id, user_id)` - deletes deck with authorization

### API Endpoints

- `DELETE /users/{user_id}/decks/{deck_id}` - Delete a deck (204 No Content on success)
- `DELETE /users/{user_id}/cards/{card_id}` - Delete a card (204 No Content on success)

Both endpoints return:

- `403 Forbidden` if user doesn't own the resource
- `404 Not Found` if resource doesn't exist
- `204 No Content` on successful deletion

### Tests

Added 6 comprehensive tests:

- `delete_deck_success` - Successful deletion
- `delete_deck_not_found` - Error when deck doesn't exist
- `delete_deck_wrong_user` - Authorization check
- `delete_card_success` - Successful deletion
- `delete_card_not_found` - Error when card doesn't exist
- `delete_card_wrong_user` - Authorization check

**Total tests: 42** (36 from Story 2 + 6 new)

### Database Behavior

When a deck is deleted:

- The deck record is removed from `decks` table
- Cards with `deck_id = <deleted_deck_id>` have their `deck_id` set to NULL
- Cards are **NOT** deleted (preserved for the user)
- This is enforced by `ON DELETE SET NULL` constraint on `cards.deck_id`

When a card is deleted:

- The card record is permanently removed from `cards` table
- Associated reviews and review_logs remain (for historical data)

## Frontend Implementation

### Frontend Domain Layer

#### Repositories Updated

1. **CardRepository** - Added `deleteCard` method

   ```dart
   AsyncResult<void> deleteCard({
     required String userId,
     required String cardId,
   });
   ```

2. **DeckRepository** - Updated `deleteDeck` signature

   ```dart
   Future<void> deleteDeck({
     required String userId,
     required String deckId,
   });
   ```

#### Frontend Use Cases Created

1. **DeleteDeck** (`lib/features/cards/domain/use_cases/delete_deck.dart`)
   - Calls repository with error handling
   - Returns Either<Failure, void>

2. **DeleteCard** (`lib/features/cards/domain/use_cases/delete_card.dart`)
   - Calls repository deleteCard
   - Returns Either<Failure, void>

### Data Layer

Implemented repository methods:

- `CardRepositoryImpl.deleteCard` - DELETE request to `/users/{userId}/cards/{cardId}`
- `DeckRepositoryImpl.deleteDeck` - DELETE request to `/users/{userId}/decks/{deckId}`

### Presentation Layer

#### Providers

Added to `deck_providers.dart`:

- `deleteDeckUseCaseProvider` - Provides DeleteDeck use case

Added to `card_providers.dart`:

- `deleteCardUseCaseProvider` - Provides DeleteCard use case

#### UI Features

**DecksScreen** (`decks_screen.dart`):

- Swipe-to-delete gesture on deck cards (Dismissible widget)
- Delete confirmation dialog with warning about cards being moved to "No Deck"
- Red background with trash icon appears during swipe
- Success/error snackbar feedback
- Auto-refreshes list after deletion

**DeckCardsScreen** (`deck_cards_screen.dart`):

- Swipe-to-delete gesture on card items
- Delete confirmation dialog showing card question
- Red background with trash icon during swipe
- Success/error snackbar feedback
- Calls `onDeleted` callback to refresh the list

### User Experience

1. **Delete Deck**:
   - Swipe left on a deck card
   - Confirmation dialog appears: "Are you sure you want to delete [deck name]? Cards in this deck will not be deleted, they will be moved to 'No Deck'."
   - If confirmed, deck is deleted and list refreshes
   - Snackbar shows "Deck [name] deleted"

2. **Delete Card**:
   - Swipe left on a card in the deck view
   - Confirmation dialog appears showing the question
   - If confirmed, card is deleted and list refreshes
   - Snackbar shows "Card deleted"

3. **Error Handling**:
   - Network errors show red snackbar with error message
   - User can retry by refreshing the list

## Testing

### Backend Tests

```bash
cd re-mem
cargo test --lib  # All 42 tests pass
```

Test coverage:

- Success scenarios for both delete operations
- Not found error handling
- Authorization error handling (wrong user)
- Repository layer tests
- Use case tests

### Frontend Tests

```bash
cd re-mem-ui
mise exec -- flutter test
```

12/13 tests pass (1 unrelated UI test failure exists from before).

## API Examples

### Delete Deck

```bash
curl -X DELETE http://localhost:3000/users/ae87b4cc-5a57-471b-9740-837f3440db6c/decks/{deck_id}
# Response: 204 No Content
```

### Delete Card

```bash
curl -X DELETE http://localhost:3000/users/ae87b4cc-5a57-471b-9740-837f3440db6c/cards/{card_id}
# Response: 204 No Content
```

## Files Modified

### Backend

- `src/application/use_cases/delete_deck.rs` (new)
- `src/application/use_cases/delete_card.rs` (new)
- `src/application/use_cases/mod.rs` (exports)
- `src/application/services.rs` (delete methods)
- `src/presentation/handlers.rs` (delete handlers)
- `src/presentation/router.rs` (DELETE routes)

### Frontend

- `lib/features/cards/domain/repositories/card_repository.dart` (deleteCard signature)
- `lib/features/cards/domain/repositories/deck_repository.dart` (deleteDeck signature)
- `lib/features/cards/domain/use_cases/delete_deck.dart` (new)
- `lib/features/cards/domain/use_cases/delete_card.dart` (new)
- `lib/features/cards/data/repositories/card_repository_impl.dart` (implementation)
- `lib/features/cards/data/repositories/deck_repository_impl.dart` (implementation)
- `lib/features/cards/presentation/providers/deck_providers.dart` (provider)
- `lib/features/cards/presentation/providers/card_providers.dart` (provider)
- `lib/features/cards/presentation/screens/decks_screen.dart` (swipe-to-delete UI)
- `lib/features/cards/presentation/screens/deck_cards_screen.dart` (swipe-to-delete UI)

## Security Considerations

- ✅ Authorization checks prevent users from deleting others' content
- ✅ User ID verified against resource ownership
- ✅ Returns 403 Forbidden for unauthorized attempts
- ✅ Deck deletion uses CASCADE NULL to preserve cards
- ✅ No orphaned data - cards remain accessible to user

## Architecture Patterns Used

- ✅ **Hexagonal Architecture**: Use cases orchestrate business logic
- ✅ **SOLID Principles**: Single responsibility for each use case
- ✅ **Clean Architecture**: Domain → Application → Infrastructure → Presentation
- ✅ **Repository Pattern**: Abstract data access
- ✅ **Either Pattern** (Frontend): Explicit error handling with fpdart
- ✅ **Authorization at Use Case Level**: Business logic enforces security

## Future Enhancements (Not Implemented)

- Undo functionality (restore deleted deck/card)
- Bulk delete operations
- Soft delete with trash/archive
- Cascade delete option (delete cards when deck is deleted)
- Delete confirmation with password/biometric for safety
