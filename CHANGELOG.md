# Changelog

All notable changes to AuthKit will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-15

### 🎉 Initial Release

AuthKit's first release provides a complete, production-ready authentication foundation for Rust applications.

### Added

#### Core Authentication
- **Single Entry Point Design** - All authentication operations through one `Auth` object
- **Framework-Agnostic Architecture** - Works with any Rust application (HTTP servers, CLI tools, workers, proxies)
- **Builder Pattern** - Intuitive `Auth::builder()` API for configuration

#### Database Support
- **SQLite Backend** - Built-in SQLite support with `:memory:` for testing
- **PostgreSQL Backend** - Production-ready Postgres support
- **Automatic Migrations** - Schema management via `auth.migrate()`
- **Connection Pooling** - Efficient connection management with SQLx

#### User Management
- **User Registration** - `auth.register()` with email and password
- **Secure Password Hashing** - Argon2id by default (bcrypt available)
- **Email Validation** - RFC 5322 compliant validation
- **Password Validation** - Configurable strength requirements (min 8 chars, mixed case, numbers, special chars)

#### Session Management
- **Database-Backed Sessions** - Persistent session storage
- **Login/Logout** - `auth.login()` and `auth.logout()` operations
- **Session Verification** - `auth.verify()` to validate session tokens
- **Automatic Expiration** - 24-hour default session lifetime
- **Secure Token Generation** - Cryptographically secure random tokens (32 bytes)

#### Email Verification ✨
- **Send Verification Tokens** - `auth.send_email_verification()` generates unique tokens
- **Verify Email Addresses** - `auth.verify_email()` validates and marks emails as verified
- **Resend Functionality** - `auth.resend_email_verification()` for users who didn't receive emails
- **Single-Use Tokens** - Tokens can only be used once for security
- **Time-Limited Tokens** - 24-hour expiration on verification tokens
- **SHA-256 Token Hashing** - Secure token storage in database

#### Security Features
- **Timing-Safe Comparisons** - Protection against timing attacks
- **Secure Random Generation** - Cryptographically secure token/ID generation
- **Password Hash Storage** - Passwords never stored in plaintext
- **SQL Injection Protection** - Parameterized queries throughout
- **Token Reuse Prevention** - Single-use enforcement on verification tokens

#### Database Schema
- **Users Table** - Email, password hash, verification status, timestamps
- **Sessions Table** - Active sessions with expiration and user references
- **Tokens Table** - Unified table for email verification, password reset, magic links (extensible)
- **Optimized Indexes** - Performance-tuned indexes on all lookups
- **Foreign Key Constraints** - Data integrity with cascade deletes

#### Developer Experience
- **Comprehensive Documentation** - Full API reference and examples
- **Type Safety** - Strong typing throughout the API
- **Clear Error Types** - Detailed error variants for all failure cases
- **Zero Configuration Defaults** - Works out of the box with secure defaults
- **Async/Await** - Built on Tokio for high-performance async operations

#### Testing
- **68 Test Cases** - Comprehensive test coverage
- **Integration Tests** - Full authentication flow tests
- **Error Handling Tests** - Edge cases and failure scenarios
- **Validation Tests** - Email and password validation coverage
- **Email Verification Tests** - Complete verification workflow tests
- **In-Memory Testing** - Fast tests with SQLite `:memory:`

#### Examples
- **Email Verification Example** - Complete workflow demonstration
- **Runnable Examples** - `cargo run --example email_verification`

### Feature Flags

- `sqlite` - SQLite database backend (enabled by default)
- `postgres` - PostgreSQL database backend
- `argon2` - Argon2id password hashing (enabled by default)
- `bcrypt` - bcrypt password hashing

### Dependencies

Core dependencies:
- `tokio` - Async runtime
- `sqlx` - Database operations
- `argon2` / `bcrypt` - Password hashing
- `sha2` - Token hashing
- `hex` - Hex encoding
- `rand` - Secure random generation
- `thiserror` - Error handling
- `async-trait` - Trait async support
- `serde` - Serialization

### API Overview

```rust
// Initialize
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;

// Register
let user = auth.register(Register {
    email: "user@example.com".into(),
    password: "SecurePass123!".into(),
}).await?;

// Send verification
let verification = auth.send_email_verification(SendEmailVerification {
    user_id: user.id.clone(),
}).await?;

// Verify email
let verified_user = auth.verify_email(VerifyEmail {
    token: verification.token,
}).await?;

// Login
let session = auth.login(Login {
    email: "user@example.com".into(),
    password: "SecurePass123!".into(),
}).await?;

// Verify session
let user = auth.verify(Verify {
    token: session.token.clone(),
}).await?;

// Logout
auth.logout(Logout {
    token: session.token,
}).await?;
```

### Documentation

- **README.md** - Complete getting started guide and API reference
- **AGENTS.md** - Architecture and contribution guidelines
- **DATABASE_ARCHITECTURE.md** - Detailed database design documentation
- **Examples** - Working code examples

### Performance

- O(1) database operations with proper indexing
- Connection pooling for optimal resource usage
- Async-first design for high concurrency
- Minimal memory footprint

### Security Considerations

AuthKit handles:
- ✅ Secure password hashing (Argon2id)
- ✅ Timing-safe password comparison
- ✅ Cryptographically secure token generation
- ✅ SQL injection prevention
- ✅ Token expiration enforcement
- ✅ Single-use token enforcement

Applications should handle:
- Rate limiting on authentication endpoints
- Email delivery for verification tokens
- HTTPS/TLS for token transmission
- CSRF protection on web endpoints
- Session token storage (secure cookies, etc.)

### Known Limitations

- Email verification is optional (by design - users can login without verification)
- No built-in email sending (application's responsibility - framework-agnostic design)
- No OAuth/SSO support yet (planned for future releases)
- No password reset flow yet (planned for v0.2.0)
- No magic link authentication yet (planned for v0.2.0)

### What's Next

Planned for v0.2.0:
- Password reset flow
- Magic link authentication
- JWT session strategy
- Refresh tokens
- Rate limiting utilities
- Audit logging

Planned for v0.3.0:
- OAuth/SSO integration
- Two-factor authentication
- Session device management
- Framework adapters (Axum, Actix, Rocket)

### Migration Guide

N/A - Initial release

### Breaking Changes

N/A - Initial release

### Contributors

- Akshay B (@Akshay2642005) - Initial implementation

### License

This project is dual-licensed under MIT and Apache-2.0.

---

## Version History

- **0.1.0** (2025-01-15) - Initial release with core authentication and email verification

[Unreleased]: https://github.com/Akshay2642005/authkit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Akshay2642005/authkit/releases/tag/v0.1.0