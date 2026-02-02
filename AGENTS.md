# AuthKit - Agent Guidelines

Rust authentication library inspired by better-auth. Framework-agnostic, SQLite/Postgres, secure defaults.

## Commands

```bash
# Build & Check
cargo build                              # Build with default features (sqlite, argon2)
cargo build --all-features               # Build with all features
cargo check                              # Fast type-check

# Test
cargo test                               # Run all tests
cargo test --lib                         # Library tests only
cargo test test_register_user_success    # Single test by name
cargo test validation_tests              # Test module by name
cargo test -- --nocapture                # Show println! output
cargo test -- --nocapture --test-threads=1  # Sequential with output

# Lint & Format
cargo fmt                                # Format (uses tab_spaces = 2)
cargo clippy                             # Lint
cargo clippy --all-features              # Lint all features
```

## Structure

```
src/
├── lib.rs              # Public exports, prelude
├── auth.rs             # Auth struct (single entry point)
├── builder.rs          # AuthBuilder pattern
├── error.rs            # AuthError enum (thiserror)
├── types.rs            # User, Session, Database, VerificationToken
├── prelude.rs          # Common imports for users
├── database/           # DatabaseTrait + SQLite/Postgres impls
├── strategies/         # Password, Session, Token strategies
├── operations/         # register, login, logout, verify, email_verification
├── validation/         # email, password validation
├── security/           # tokens, timing-safe operations
└── tests/              # Unit + integration tests
```

## Code Style

### Formatting
- **2-space indentation** (see `.rustfmt.toml`)
- Run `cargo fmt` before committing

### Imports
```rust
// Order: crate imports first, then external
use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::User;

use async_trait::async_trait;
use sqlx::SqlitePool;
```

### Error Handling
- Use `crate::error::Result<T>` (alias for `std::result::Result<T, AuthError>`)
- Use `thiserror` for error variants
- Use `?` operator, avoid `.unwrap()` in non-test code
- Add new error variants to `src/error.rs`

### Naming
- Types: `PascalCase` (User, Session, AuthBuilder)
- Functions/methods: `snake_case` (find_user_by_email)
- Constants: `SCREAMING_SNAKE_CASE`
- Feature flags: `lowercase-kebab` in Cargo.toml

### Visibility
- Public API: Only `Auth`, types in `prelude.rs`, and operation structs
- Internal: Use `pub(crate)` for cross-module access
- Traits like `DatabaseTrait`, `PasswordStrategy` are `pub(crate)` - NEVER expose

## Architecture Rules

### Single Entry Point
Users interact ONLY with `Auth`. No repositories, no generics, no leaked internals.

### Strategy Pattern (CRITICAL)
Strategies receive database as parameter, never store it:

```rust
// ✅ CORRECT - Pass db as parameter
impl SomeStrategy {
    async fn do_something(&self, db: &dyn DatabaseTrait, ...) -> Result<()> {
        db.some_operation(...).await
    }
}

// ❌ WRONG - Don't store database
pub struct SomeStrategy {
    db: Arc<Box<dyn DatabaseTrait>>,  // NEVER do this
}
```

### Adding Database Features
1. Add method to `DatabaseTrait` in `src/database/mod.rs`
2. Implement in `src/database/sqlite.rs` (use `?` placeholders)
3. Implement in `src/database/postgres.rs` (use `$1, $2` placeholders)
4. Update migrations in both if schema changes

### Feature Flags
```toml
default = ["sqlite", "argon2"]
sqlite = ["sqlx/sqlite"]      # SQLite backend
postgres = ["sqlx/postgres"]  # Postgres backend
argon2 = []                   # Argon2id password hashing
bcrypt = []                   # bcrypt (not yet implemented)
jwt = []                      # JWT tokens (future)
email-queue = []              # Background email jobs
```

## Testing

### Test Helper
```rust
use crate::tests::integration_tests::setup_test_auth;

#[tokio::test]
async fn test_something() {
    let auth = setup_test_auth().await.unwrap();  // In-memory SQLite
    // test code...
}
```

### Conventions
- Email: `test@example.com`, `duplicate@example.com`
- Password: `SecurePass123` (meets all requirements)
- Each test uses isolated in-memory database
- Test both success AND failure paths

## Anti-Patterns (NEVER)

❌ Expose `DatabaseTrait` or strategy traits publicly
❌ Store database references in strategies
❌ Add framework-specific logic to core
❌ Leak SQLx types into public API
❌ Use `.unwrap()` in non-test code
❌ Introduce global state
❌ Create compile-time-inconsistent patterns between SQLite/Postgres

## Security Rules

- Argon2id for password hashing (default)
- Timing-safe comparisons for tokens (use `subtle` crate)
- Sessions expire in 24h by default
- Tokens use cryptographic randomness
- Validate email format and password strength before processing

## Key Types

```rust
pub struct Auth { /* Arc<AuthInner> */ }
pub struct User { id, email, created_at, email_verified, email_verified_at }
pub struct Session { token, user_id, expires_at }
pub enum AuthError { UserNotFound, InvalidCredentials, WeakPassword, ... }
```

## Adding New Operations

1. Create `src/operations/my_operation.rs`
2. Add operation struct: `pub struct MyOperation { ... }`
3. Implement `Auth::my_operation(&self, req: MyOperation) -> Result<...>`
4. Re-export in `src/operations/mod.rs` and `src/lib.rs`
5. Add to prelude if commonly used

## Internal Docs

- `internal/` - Development roadmaps, architecture notes (excluded from crate)
- `docs/EMAIL_INTEGRATION.md` - Email sender implementation guide
