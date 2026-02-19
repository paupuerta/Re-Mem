# API Documentation

## Overview

ReMem API provides REST endpoints for managing flashcards, reviews, and language learning progress. The API follows OpenAPI 3.0 specification.

## Base URL

```
http://localhost:3000/api/v1
```

## Authentication

Currently in MVP phase. Authentication to be implemented in Phase 2.

## Response Format

All responses are JSON. Successful responses include a 2xx status code.

### Success Response

```json
{
    "id": "uuid",
    "data": {}
}
```

### Error Response

```json
{
    "error": "Error message",
    "status": 400,
    "details": "Additional context"
}
```

## Endpoints

### Users

#### Create User
```
POST /users
Content-Type: application/json

{
    "email": "user@example.com",
    "name": "John Doe"
}

Response: 201 Created
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe"
}
```

#### Get User
```
GET /users/{user_id}

Response: 200 OK
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe"
}
```

### Cards (Flashcards)

#### Create Card
```
POST /users/{user_id}/cards
Content-Type: application/json

{
    "question": "What is the capital of France?",
    "answer": "Paris"
}

Response: 201 Created
{
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "question": "What is the capital of France?",
    "answer": "Paris"
}
```

#### List User Cards
```
GET /users/{user_id}/cards

Response: 200 OK
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "question": "What is the capital of France?",
        "answer": "Paris"
    }
]
```

### Reviews (Study Sessions)

#### Submit Review
```
POST /users/{user_id}/cards/{card_id}/reviews
Content-Type: application/json

{
    "grade": 4
}

Grade scale (FSRS):
- 0: Again (forgot)
- 1: Hard
- 2: Good
- 3: Easy
- 4: Very Easy
- 5: Perfect

Response: 201 Created
{
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "card_id": "550e8400-e29b-41d4-a716-446655440001",
    "grade": 4
}
```

## Error Codes

| Code | Meaning | Example |
|------|---------|---------|
| 400 | Bad Request | Invalid input validation |
| 401 | Unauthorized | Missing/invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | User/Card doesn't exist |
| 409 | Conflict | Duplicate email |
| 500 | Server Error | Unexpected error |

## Health Check

```
GET /health

Response: 200 OK
{
    "status": "ok"
}
```

## Rate Limiting

To be implemented in Phase 2.

## Webhooks

To be implemented in future phase for event notifications.

## Changelog

### v0.1.0 (MVP)
- Basic CRUD operations for Users
- Card management
- Review submission with FSRS integration (planned)
- Health check endpoint

---

For full interactive documentation, visit `/docs/swagger-ui` when server is running.
