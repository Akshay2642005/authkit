# AuthKit

**A better-auth–inspired authentication library for Rust**  
Plug-and-play. Framework-agnostic. Opinionated, but extensible.

---

## Overview

AuthKit is a Rust authentication library designed to feel like [better-auth](https://github.com/better-auth/better-auth), but for the Rust ecosystem.

It provides:

- **A single `Auth` entry point**
- **Opinionated defaults** (secure by default)
- **Zero framework lock-in**
- **Database-backed authentication** using SQLx (Postgres / SQLite)
- **The same API** across HTTP servers, CLIs, background workers, and proxies

AuthKit is **not** a framework, middleware, or ORM.  
It is a self-contained authentication engine that you embed into your application.

---

## Design Goals

AuthKit is built around the following **non-negotiable principles**:

### 1. Single Entry Point

Users interact with **one object only**: `Auth`.

```rust
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;
```

- No repositories.
- No generics.
- No leaked internals.

### 2. Framework-Agnostic by Design

AuthKit:

- Does **not** depend on Axum, Actix, Rocket, Hyper, or Tower
- Does **not** assume HTTP
- Works equally well in:
  - CLI tools
  - REST APIs
  - gRPC services
  - Proxies (Pingora)
  - Background workers

**Framework adapters live outside the core.**

### 3. Opinionated Defaults, Explicit Overrides

AuthKit ships with:

- Argon2id password hashing
- Database-backed sessions
- Secure token generation
- Sensible expiry defaults

Users can override behavior explicitly, but **never accidentally weaken security**.

### 4. No Leaky Abstractions

AuthKit hides:

- SQLx
- Database schemas
- Crypto implementations
- Token formats

Users **never interact with**:

- Traits
- Repositories
- Lifetimes
- Generic parameters

### 5. Same API Everywhere

```rust
auth.register(Register { ... }).await?;
auth.login(Login { ... }).await?;
auth.verify(Verify { token }).await?;
auth.logout(Logout { token }).await?;
```

These calls behave **identically** whether invoked from:

- an HTTP handler
- a CLI command
- a test
- a background task

---

## Non-Goals

AuthKit **intentionally does not** attempt to:

- Be an OAuth provider (may integrate later)
- Replace application authorization logic
- Act as a user management UI
- Tie itself to any web framework

---

## Example Usage

### Create an Auth Instance

```rust
use authkit::prelude::*;

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;
```

### Register a User

```rust
auth.register(Register {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;
```

### Login

```rust
let session = auth.login(Login {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;
```

### Verify a Session

```rust
let user = auth.verify(Verify {
    token: session.token.clone(),
}).await?;
```

### Logout

```rust
auth.logout(Logout {
    token: session.token,
}).await?;
```

---

## Architecture

### High-Level Structure

```
Auth
 └── AuthInner (Arc)
     ├── Database (Arc<Box<dyn DatabaseTrait>>)
     ├── PasswordStrategy (Box<dyn PasswordStrategy>)
     ├── SessionStrategy (Box<dyn SessionStrategy>)
     └── TokenStrategy (Box<dyn TokenStrategy>)
```

**Key characteristics:**

- `Auth` is cheap to clone (uses `Arc` internally)
- Internals are completely hidden
- Components are swappable via builder pattern
- No global state

### Strategy Pattern (IMPORTANT!)

All strategies follow a **consistent pattern** where the database is passed as a parameter, not stored:

```rust
// ❌ WRONG - Don't store database in strategy
pub struct SomeStrategy {
    db: Arc<Box<dyn DatabaseTrait>>,
}

// ✅ CORRECT - Pass database as parameter
pub struct SomeStrategy;

#[async_trait]
impl SomeTrait for SomeStrategy {
    async fn do_something(
        &self,
        db: &dyn DatabaseTrait,  // ✅ Database passed as parameter
        ...
    ) -> Result<()> {
        db.some_operation(...).await
    }
}
```

**Why this pattern?**

1. **Consistency** - All strategies work the same way
2. **No coupling** - Strategies don't own database connections
3. **Testability** - Easy to mock `DatabaseTrait`
4. **Flexibility** - Strategies remain stateless

### Full Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  (Auth, Operations: register, login, verify, etc.)      │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ↓
┌─────────────────────────────────────────────────────────┐
│                  Strategy Layer                          │
│  • PasswordStrategy   (Argon2, Bcrypt, etc.)            │
│  • SessionStrategy    (Database-backed)                  │
│  • TokenStrategy      (Database-backed)                  │
│                                                           │
│  Strategies receive &dyn DatabaseTrait as parameter      │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ↓
┌─────────────────────────────────────────────────────────┐
│              DatabaseTrait (Abstraction)                 │
│                                                           │
│  User Operations:                                        │
│    • find_user_by_email()                                │
│    • find_user_by_id()                                   │
│    • create_user()                                       │
│                                                           │
│  Session Operations:                                     │
│    • create_session()                                    │
│    • find_session()                                      │
│    • delete_session()                                    │
│    • delete_expired_sessions()                           │
│                                                           │
│  Token Operations:                                       │
│    • create_token()                                      │
│    • find_token()                                        │
│    • mark_token_used()                                   │
│    • delete_token()                                      │
│    • delete_expired_tokens()                             │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ↓
┌─────────────────────────────────────────────────────────┐
│            Backend Implementations                       │
│                                                           │
│  ┌─────────────────┐        ┌─────────────────┐         │
│  │ SqliteDatabase  │        │ PostgresDatabase│         │
│  │                 │        │                 │         │
│  │ Uses ? params   │        │ Uses $N params  │         │
│  │ SQLite pool     │        │ Postgres pool   │         │
│  └─────────────────┘        └─────────────────┘         │
└─────────────────────────────────────────────────────────┘
```

---

## Database Support

Currently supported:

- **SQLite**
- **PostgreSQL**

Backed by SQLx, but **SQLx is not exposed**.

AuthKit manages:

- Schema
- Migrations
- Versioning

```rust
auth.migrate().await?;
```

### Database Schema

- **`users`** - User accounts with email, password, verification status
- **`sessions`** - Active user sessions with expiration
- **`tokens`** - Unified table for email verification, password reset, magic links, etc.

All tables include:
- Proper indexes for performance
- Foreign key constraints for data integrity
- Automatic cleanup of expired data

---

## Security Defaults

| Feature                | Default              |
|------------------------|----------------------|
| Password hashing       | Argon2id             |
| Timing-safe compares   | ✅ Enabled           |
| Session expiration     | ✅ Enabled (24h)     |
| Token entropy          | High (crypto secure) |
| Password reuse         | 🚫 Prevented         |
| Weak passwords         | 🚫 Rejected          |

Security-sensitive behavior requires **explicit opt-out**.

---

## Feature Flags

```toml
[features]
default = ["sqlite", "argon2"]

# Database backends
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]

# Password strategies
argon2 = []
bcrypt = []

# Token strategies (future)
jwt = []
```

---

## Adapters

Adapters translate framework-specific concepts into AuthKit calls.

**Planned adapters:**

- Axum
- Actix
- Rocket
- Hyper
- Pingora
- CLI helpers

**Important:** Adapters contain **no authentication logic**.  
They only translate request/response formats.

---

## Project Status

**Current phase:** Foundation ✅

### Implemented:

- ✅ Core Auth API
- ✅ Builder pattern
- ✅ SQLite backend
- ✅ PostgreSQL backend
- ✅ Argon2 password hashing
- ✅ Database sessions
- ✅ Token infrastructure (strategy + database methods)
- ✅ Email verification flow (send, verify, resend operations)

### Planned:

- 🔜 Password reset flow
- 🔜 Magic link authentication
- 🔜 JWT sessions
- 🔜 Refresh tokens
- 🔜 Axum adapter
- 🔜 Actix adapter
- 🔜 Rate limiting
- 🔜 Audit logging
- 🔜 OAuth integration
- 🔜 Two-factor authentication

---

## Contribution Guidelines (Agents)

If you are contributing to this project:

### You MUST:

✅ **Preserve the single-entry-point design**  
✅ **Avoid exposing generics or traits publicly**  
✅ **Keep framework dependencies out of core**  
✅ **Prefer composition over configuration**  
✅ **Default to secure behavior**  
✅ **Follow the strategy pattern** (pass database as parameter, don't store it)  
✅ **Add database methods to `DatabaseTrait`**, not to strategies  
✅ **Implement all database methods in both SQLite and Postgres**  
✅ **Group related database methods** by feature area (users, sessions, tokens, etc.)

### You MUST NOT:

❌ **Add framework-specific logic to core**  
❌ **Leak SQLx types into the public API**  
❌ **Introduce global state**  
❌ **Require users to wire repositories manually**  
❌ **Store database references in strategies**  
❌ **Expose `DatabaseTrait` or strategy traits publicly**  
❌ **Create inconsistent patterns** (all strategies must follow the same approach)

### Key Principle:

**If a change makes the API feel less like better-auth, it is probably wrong.**

---

## Adding New Features

### To Add a New Database Feature:

1. **Add methods to `DatabaseTrait`** in `src/database/mod.rs`
2. **Implement for SQLite** in `src/database/sqlite.rs` (use `?` placeholders)
3. **Implement for Postgres** in `src/database/postgres.rs` (use `$N` placeholders)
4. **Update migrations** in both backends if schema changes needed
5. **Use in strategies** by passing `db: &dyn DatabaseTrait` as parameter

**Example:**

```rust
// 1. Add to DatabaseTrait
#[async_trait]
pub(crate) trait DatabaseTrait: Send + Sync {
    async fn create_password_reset(&self, user_id: &str, token_hash: &str) -> Result<()>;
}

// 2. Use in strategy (don't store db!)
pub struct PasswordResetStrategy;

impl PasswordResetStrategy {
    async fn create_token(
        &self,
        db: &dyn DatabaseTrait,  // ✅ Passed as parameter
        user_id: &str,
    ) -> Result<String> {
        let token = generate_token();
        db.create_password_reset(user_id, &hash_token(&token)).await?;
        Ok(token)
    }
}
```

### For Detailed Instructions:

See **[docs/DATABASE_ARCHITECTURE.md](docs/DATABASE_ARCHITECTURE.md)** for comprehensive guidance on:

- Database architecture details
- Step-by-step feature addition guide
- Common patterns and best practices
- SQL dialect differences (SQLite vs Postgres)
- Testing strategies

---

## Internal Documentation

- 📚 **[docs/DATABASE_ARCHITECTURE.md](docs/DATABASE_ARCHITECTURE.md)** - Detailed database architecture and how to add features
- 📋 **[README.md](README.md)** - Public-facing documentation and API reference

---

## License

This project is dual-licensed under your choice of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

**Remember:** AuthKit's strength comes from its simplicity and consistency.  
Every feature should enhance, not complicate, the developer experience.