# AuthKit

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

A better-auth–inspired authentication library for Rust. Plug-and-play, framework-agnostic, and opinionated yet extensible.

## Overview

AuthKit is a Rust authentication library designed to feel like [better-auth](https://github.com/better-auth/better-auth), but for the Rust ecosystem. It provides secure, database-backed authentication with a single, unified API that works across any context—HTTP servers, CLI tools, background workers, or proxies.

### Key Features

- **🔒 Secure by Default** - Argon2id password hashing, timing-safe comparisons, secure token generation
- **🎯 Single Entry Point** - One `Auth` object for all authentication operations
- **🔧 Framework-Agnostic** - Works with Axum, Actix, Rocket, or standalone—no framework lock-in
- **💾 Database-Backed** - SQLite and PostgreSQL support via SQLx (hidden from API)
- **🚀 Simple API** - Register, login, verify, logout—same API everywhere
- **⚡ Async-First** - Built on Tokio for high-performance async operations
- **🎨 Extensible** - Swappable password and session strategies

## Quick Start

### Installation

Add AuthKit to your `Cargo.toml`:

```toml
[dependencies]
authkit = "0.1"
tokio = { version = "1.28", features = ["full"] }
```

### Basic Usage

```rust
use authkit::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an Auth instance
    let auth = Auth::builder()
        .database(Database::sqlite("auth.db").await?)
        .build()?;

    // Run migrations
    auth.migrate().await?;

    // Register a new user
    let user = auth.register(Register {
        email: "user@example.com".into(),
        password: "secure-password".into(),
    }).await?;

    println!("User registered: {}", user.email);

    // Login
    let session = auth.login(Login {
        email: "user@example.com".into(),
        password: "secure-password".into(),
    }).await?;

    println!("Session token: {}", session.token);

    // Verify a session
    let user = auth.verify(Verify {
        token: session.token.clone(),
    }).await?;

    println!("Verified user: {}", user.email);

    // Logout
    auth.logout(Logout {
        token: session.token,
    }).await?;

    println!("Logged out successfully");

    Ok(())
}
```

## Design Philosophy

AuthKit is built around five core principles:

### 1. Single Entry Point

Users interact with only one object: `Auth`. No repositories, no generics, no leaked internals.

```rust
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;
```

### 2. Framework-Agnostic by Design

AuthKit doesn't depend on Axum, Actix, Rocket, Hyper, or Tower. It works equally well in:

- REST APIs
- CLI tools
- gRPC services
- Proxies (Pingora)
- Background workers

Framework adapters live outside the core library.

### 3. Opinionated Defaults, Explicit Overrides

Ships with secure defaults:
- Argon2id password hashing
- Database-backed sessions
- Secure token generation
- Sensible expiry defaults

Override behavior explicitly when needed, but never accidentally weaken security.

### 4. No Leaky Abstractions

AuthKit hides implementation details:
- SQLx internals
- Database schemas
- Crypto implementations
- Token formats

Users never interact with traits, repositories, lifetimes, or generic parameters in the public API.

### 5. Same API Everywhere

```rust
auth.register(Register { ... }).await?;
auth.login(Login { ... }).await?;
auth.verify(Verify { ... }).await?;
auth.logout(Logout { ... }).await?;
```

These calls behave identically whether invoked from an HTTP handler, CLI command, test, or background task.

## Architecture

```
Auth
 └── AuthInner (Arc)
     ├── Database (trait object)
     ├── PasswordStrategy
     ├── SessionStrategy
     └── TokenStrategy
```

**Key characteristics:**
- `Auth` is cheap to clone (uses `Arc` internally)
- Internals are completely hidden
- Components are swappable via builder pattern
- No global state required

### Strategy Pattern

AuthKit uses a consistent strategy pattern for all authentication components:

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
│  • User Operations (create, find)                        │
│  • Session Operations (create, find, delete)             │
│  • Token Operations (create, verify, mark used)          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ↓
┌─────────────────────────────────────────────────────────┐
│            Backend Implementations                       │
│  • SqliteDatabase   (SQLite with ? params)              │
│  • PostgresDatabase (Postgres with $N params)           │
└─────────────────────────────────────────────────────────┘
```

**Design Benefits:**
- Strategies remain stateless and don't store database references
- Database logic is centralized in backend implementations
- Easy to add new database backends (MySQL, etc.)
- Easy to mock for testing
- No SQLx types leak into public API

For detailed architecture documentation, see [docs/DATABASE_ARCHITECTURE.md](docs/DATABASE_ARCHITECTURE.md)

## Feature Flags

AuthKit uses Cargo features for optional functionality:

```toml
[features]
default = ["sqlite", "argon2"]

# Database backends
sqlite = ["sqlx/sqlite", "sqlx/runtime-tokio"]
postgres = ["sqlx/postgres", "sqlx/runtime-tokio"]

# Password hashing strategies
argon2 = ["dep:argon2", "dep:password-hash"]
bcrypt = ["dep:bcrypt"]

# Token strategies
jwt = ["dep:jsonwebtoken"]
```

### Examples

**PostgreSQL with Argon2:**
```toml
authkit = { version = "0.1", default-features = false, features = ["postgres", "argon2"] }
```

**SQLite with bcrypt:**
```toml
authkit = { version = "0.1", default-features = false, features = ["sqlite", "bcrypt"] }
```

## Database Support

### SQLite

```rust
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;
```

### PostgreSQL

```rust
let auth = Auth::builder()
    .database(Database::postgres("postgresql://user:pass@localhost/authdb").await?)
    .build()?;
```

### Migrations

AuthKit manages its own schema and migrations:

```rust
auth.migrate().await?;
```

**Database Schema:**
- `users` - User accounts with email and password
- `sessions` - Active user sessions
- `tokens` - Unified table for email verification, password reset, etc.

All tables include proper indexes and foreign key constraints for optimal performance and data integrity.

## API Reference

### Auth Operations

#### Register

Create a new user account:

```rust
let user = auth.register(Register {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;
```

**Validation:**
- Email must be valid format
- Password must meet minimum security requirements
- Email must be unique

#### Login

Authenticate and create a session:

```rust
let session = auth.login(Login {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;
```

**Returns:**
- `Session` with token, user_id, and expiration

#### Verify

Verify a session token and retrieve user:

```rust
let user = auth.verify(Verify {
    token: session_token,
}).await?;
```

**Errors:**
- Invalid token
- Expired session
- Session not found

#### Logout

Invalidate a session:

```rust
auth.logout(Logout {
    token: session_token,
}).await?;
```

#### Send Email Verification

Generate a verification token for a user:

```rust
let verification = auth.send_email_verification(SendEmailVerification {
    user_id: user.id.clone(),
}).await?;

// Send the token via email (application's responsibility)
send_email(&verification.email, &verification.token).await?;
```

**Returns:**
- `VerificationToken` with token, email, and expiration time
- Token expires in 24 hours

**Errors:**
- User not found
- Email already verified

#### Verify Email

Verify a user's email using a token:

```rust
let verified_user = auth.verify_email(VerifyEmail {
    token: verification_token,
}).await?;
```

**Returns:**
- Updated `User` with `email_verified` set to `true`

**Errors:**
- Invalid or expired token
- Token already used
- Email already verified

#### Resend Email Verification

Resend verification token to a user:

```rust
let verification = auth.resend_email_verification(ResendEmailVerification {
    email: "user@example.com".into(),
}).await?;
```

**Returns:**
- New `VerificationToken` (old tokens remain valid until used or expired)

**Errors:**
- User not found
- Email already verified

### Types

#### User

```rust
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: i64,
    pub email_verified: bool,
    pub email_verified_at: Option<i64>,
}
```

#### Session

```rust
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}
```

#### VerificationToken

```rust
pub struct VerificationToken {
    pub token: String,
    pub email: String,
    pub expires_at: i64,
}
```

## Security

### Default Security Features

| Feature | Default |
|---------|---------|
| Password hashing | Argon2id |
| Timing-safe compares | ✅ Enabled |
| Session expiration | ✅ Enabled (24 hours) |
| Token entropy | High (cryptographically secure) |
| Password reuse | 🚫 Prevented |
| Weak passwords | 🚫 Rejected |

### Password Requirements

- Minimum length: 8 characters
- Must contain at least one uppercase letter
- Must contain at least one lowercase letter
- Must contain at least one number
- Must contain at least one special character

### Email Validation

- RFC 5322 compliant email validation
- Checks for valid format and structure

## Advanced Configuration

### Custom Password Strategy

```rust
use authkit::prelude::*;
use authkit::strategies::password::PasswordStrategyType;

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .password_strategy(PasswordStrategyType::Argon2)
    .build()?;
```

### Custom Session Strategy

```rust
use authkit::prelude::*;
use authkit::strategies::session::SessionStrategyType;

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .session_strategy(SessionStrategyType::Database)
    .build()?;
```

## Error Handling

AuthKit provides a comprehensive error type:

```rust
use authkit::prelude::*;

match auth.login(login_request).await {
    Ok(session) => println!("Login successful"),
    Err(AuthError::InvalidCredentials) => println!("Wrong email or password"),
    Err(AuthError::UserNotFound) => println!("User doesn't exist"),
    Err(e) => println!("Error: {}", e),
}
```

## Examples

Check the `examples/` directory for more usage examples:

- `email_verification.rs` - Complete email verification workflow
- `basic.rs` - Simple registration and login flow (planned)
- `web_server.rs` - Integration with Axum (planned)
- `cli_tool.rs` - CLI authentication example (planned)

Run an example:

```bash
cargo run --example email_verification --features sqlite
```

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with all features:

```bash
cargo test --all-features
```

## Roadmap

### Current Status: Foundation Phase ✅

**Implemented:**
- ✅ Core Auth API
- ✅ Builder pattern
- ✅ SQLite backend
- ✅ PostgreSQL backend
- ✅ Argon2 password hashing
- ✅ Database sessions
- ✅ Email validation
- ✅ Password validation
- ✅ Token system (database-backed)
- ✅ Email verification flow (send, verify, resend)

**Planned:**
- 🔜 JWT sessions
- 🔜 Refresh tokens
- 🔜 Password reset flow
- 🔜 Magic link authentication
- 🔜 Axum adapter
- 🔜 Actix adapter
- 🔜 Rate limiting
- 🔜 Audit logging
- 🔜 OAuth integration
- 🔜 Two-factor authentication

## Contributing

Contributions are welcome! Please read our [contribution guidelines](AGENTS.md) first.

### For Contributors (Including AI Agents)

When contributing to AuthKit, you **MUST**:

- ✅ Preserve the single-entry-point design
- ✅ Avoid exposing generics or traits publicly
- ✅ Keep framework dependencies out of core
- ✅ Prefer composition over configuration
- ✅ Default to secure behavior

You **MUST NOT**:

- ❌ Add framework-specific logic to core
- ❌ Leak SQLx types into the public API
- ❌ Introduce global state
- ❌ Require users to wire repositories manually

**If a change makes the API feel less like better-auth, it's probably wrong.**

## License

This project is dual-licensed under your choice of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

Inspired by [better-auth](https://github.com/better-auth/better-auth) - an excellent authentication library for JavaScript/TypeScript.

## Internal Documentation

For contributors and maintainers:

- 📚 [Database Architecture Guide](docs/DATABASE_ARCHITECTURE.md) - Detailed guide on adding database features
- 📋 [Agent Guidelines](AGENTS.md) - Contribution guidelines for developers and AI agents

## Support

- 📖 [Documentation](https://docs.rs/authkit)
- 🐛 [Issue Tracker](https://github.com/Akshay2642005/authkit/issues)
- 💬 [Discussions](https://github.com/Akshay2642005/authkit/discussions)

---

**Built with ❤️ by [Akshay B](mailto:akshay2642005@gmail.com)**
