# Rabbit Todo API

A production-grade RESTful Todo API built with Rust, featuring multi-user support, group management, and a sophisticated points system. Built with Axum framework, implementing best practices including dependency injection via traits, unified JSON responses, and comprehensive error handling.

## Features

### Core Features
- **Multi-user Support**: Each user has their own account with points tracking
- **Group Management**: Users can create and join groups, admins can manage membership
- **Todo Management**: Full CRUD operations with rich metadata
  - Status tracking: Active, In Progress, Closed
  - Priority levels: Low, Medium, High, Critical
  - Notes and descriptions
  - Duration tracking (estimated and actual)
  - Due dates with overdue detection
- **User Assignment**: Assign multiple users to group todos
- **Points System**: Gamification with automatic point calculation
  - +2 points when closing a todo
  - -2 points when reopening a closed todo
- **Statistics & Reporting**: Aggregated status counts and durations per group

### Technical Features
- **Basic Authentication**: HTTP Basic Auth interceptor for all protected endpoints
- **Session Caching**: In-memory session cache using Moka
- **Unified JSON Response**: All responses follow a consistent structure
- **Global Error Mapping**: Comprehensive error handling with proper HTTP status codes
- **Dependency Injection**: Repository pattern with trait-based DI
- **Database**: SQLite with SQLx for type-safe queries
- **RESTful API**: Clean, predictable endpoint structure

## Architecture

```
src/
├── domain/           # Domain models and repository traits
│   ├── models.rs     # Core entities and DTOs
│   └── repositories.rs
├── infrastructure/   # Implementation layer
│   ├── database.rs   # Database setup and migrations
│   ├── error.rs      # Error types and handling
│   ├── repositories/ # Repository implementations
│   └── session_cache.rs
├── middleware/       # HTTP middleware
│   └── auth.rs       # Authentication interceptor
├── api/              # HTTP API layer
│   ├── handlers/     # Request handlers
│   ├── response.rs   # Unified response format
│   └── mod.rs        # Router configuration
└── main.rs           # Application entry point
```

## Installation

### Prerequisites
- Rust 1.70 or higher
- SQLite 3

### Build from Source

```bash
# Clone the repository
git clone https://github.com/kasunsampath-zak/rabbit-todo-api.git
cd rabbit-todo-api

# Build the project
cargo build --release

# Run the server
cargo run --release
```

### Configuration

Set environment variables:

```bash
# Database URL (default: sqlite://todos.db)
export DATABASE_URL="sqlite://todos.db"

# Server bind address (default: 0.0.0.0:3000)
export BIND_ADDRESS="0.0.0.0:3000"

# Logging level (default: info)
export RUST_LOG="rabbit_todo_api=debug"
```

## API Documentation

### Authentication

All endpoints (except user creation) require HTTP Basic Authentication:

```bash
curl -u username:password http://localhost:3000/api/users
```

### Unified Response Format

All API responses follow this structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On error:

```json
{
  "success": false,
  "data": null,
  "error": "Error message here"
}
```

### Endpoints

#### User Management

**Create User** (No auth required)
```bash
POST /api/users
Content-Type: application/json

{
  "username": "john_doe",
  "password": "secure_password",
  "is_admin": false
}
```

**List All Users**
```bash
GET /api/users
Authorization: Basic <base64(username:password)>
```

**Get User**
```bash
GET /api/users/{id}
```

**Update User** (Users can update own profile, admins can update any)
```bash
PUT /api/users/{id}
Content-Type: application/json

{
  "username": "new_username",
  "password": "new_password"
}
```

**Delete User** (Admin only)
```bash
DELETE /api/users/{id}
```

**Get User Profile** (Includes points and todo statistics)
```bash
GET /api/users/{id}/profile
```

Response:
```json
{
  "success": true,
  "data": {
    "id": "...",
    "username": "john_doe",
    "is_admin": false,
    "points": 42,
    "total_todos": 15,
    "active_todos": 5,
    "in_progress_todos": 3,
    "closed_todos": 7,
    "created_at": "2024-01-01T00:00:00Z"
  },
  "error": null
}
```

#### Group Management

**Create Group**
```bash
POST /api/groups
Content-Type: application/json

{
  "name": "Engineering Team",
  "description": "Backend engineering tasks"
}
```

**List All Groups**
```bash
GET /api/groups
```

**Get Group**
```bash
GET /api/groups/{id}
```

**Update Group** (Creator or admin only)
```bash
PUT /api/groups/{id}
Content-Type: application/json

{
  "name": "Updated Name",
  "description": "Updated description"
}
```

**Delete Group** (Creator or admin only)
```bash
DELETE /api/groups/{id}
```

**Get Group Members**
```bash
GET /api/groups/{id}/members
```

**Add Member** (Creator or admin only)
```bash
POST /api/groups/{group_id}/members/{user_id}
```

**Remove Member (Kick)** (Admin or group creator only)
```bash
DELETE /api/groups/{group_id}/members/{user_id}
```

**Get Group Statistics** (Aggregated counts and durations)
```bash
GET /api/groups/{id}/stats
```

Response:
```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "group_name": "Engineering Team",
    "active_count": 10,
    "in_progress_count": 5,
    "closed_count": 20,
    "total_estimated_duration": 120,
    "total_actual_duration": 95,
    "overdue_count": 2
  },
  "error": null
}
```

#### Todo Management

**Create Todo**
```bash
POST /api/todos
Content-Type: application/json

{
  "title": "Implement authentication",
  "description": "Add JWT authentication to API",
  "priority": "High",
  "notes": "Use RS256 algorithm",
  "estimated_duration": 8,
  "due_date": "2024-12-31T23:59:59Z",
  "group_id": "..."
}
```

**Get Todo**
```bash
GET /api/todos/{id}
```

**Update Todo**
```bash
PUT /api/todos/{id}
Content-Type: application/json

{
  "title": "Updated title",
  "status": "InProgress",
  "priority": "Critical",
  "actual_duration": 10
}
```

Note: Changing status to "Closed" awards +2 points to all assigned users. Reopening (changing from "Closed" to another status) deducts -2 points.

**Delete Todo** (Creator or admin only)
```bash
DELETE /api/todos/{id}
```

**List Group Todos**
```bash
GET /api/groups/{group_id}/todos
```

**List User Todos**
```bash
GET /api/users/{user_id}/todos
```

**Assign User to Todo**
```bash
POST /api/todos/{todo_id}/assign/{user_id}
```

**Unassign User from Todo**
```bash
DELETE /api/todos/{todo_id}/assign/{user_id}
```

**Get Assigned Users**
```bash
GET /api/todos/{id}/assigned-users
```

### Status Values
- `Active` - Todo is created but not started
- `InProgress` - Todo is being worked on
- `Closed` - Todo is completed

### Priority Values
- `Low`
- `Medium`
- `High`
- `Critical`

## Points System

The gamification system automatically tracks user contributions:

- **+2 points**: Awarded to all assigned users when a todo is marked as "closed"
- **-2 points**: Deducted from all assigned users when a closed todo is reopened

Points are visible in the user profile endpoint.

## Development

### Running Tests

```bash
cargo test
```

### Code Format

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Security Considerations

1. **Authentication**: Uses HTTP Basic Auth. In production, consider using HTTPS/TLS
2. **Password Hashing**: Uses bcrypt with default cost factor
3. **Authorization**: Implements role-based access control (admin vs. regular user)
4. **SQL Injection**: Protected via SQLx parameterized queries
5. **Session Management**: In-memory cache with TTL expiration

## Error Handling

The API uses HTTP status codes appropriately:

- `200 OK`: Successful request
- `400 Bad Request`: Invalid input or validation error
- `401 Unauthorized`: Missing or invalid authentication
- `403 Forbidden`: Authenticated but not authorized
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server-side error

All errors include a descriptive message in the response.

## Database Schema

The SQLite database includes:

- `users`: User accounts with points tracking
- `groups`: Group definitions
- `todos`: Todo items with rich metadata
- `group_members`: Many-to-many relationship between users and groups
- `todo_assignments`: Many-to-many relationship between users and todos
- `sessions`: Session storage
- `todo_status_history`: Audit trail for status changes (for points calculation)

## Performance

- In-memory session caching reduces database load
- SQLite with proper indexing for fast queries
- Connection pooling via SQLx
- Efficient async/await with Tokio runtime

## Future Enhancements

Potential improvements:
- JWT token authentication
- Real-time updates via WebSockets
- Todo attachments/file uploads
- Comments on todos
- Activity feed
- Email notifications
- Search and filtering
- Pagination for large lists
- Rate limiting
- API versioning

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.