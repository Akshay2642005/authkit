# AuthKit Testing Guide

## Overview

AuthKit includes a comprehensive test suite with **56 tests** covering all core functionality, edge cases, and security concerns.

## Quick Start

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module
cargo test validation_tests
cargo test integration_tests
cargo test error_tests
```

## Test Coverage Summary

### ✅ Core Authentication Flow
- User registration with validation
- User login with password verification
- Session creation and management
- Session verification
- Logout and session invalidation
- Multiple concurrent sessions per user

### ✅ Input Validation
- Email format validation (RFC 5322 compliance)
- Password strength requirements:
  - Minimum 8 characters
  - Maximum 128 characters
  - At least one uppercase letter
  - At least one lowercase letter
  - At least one digit
- Special characters and unicode support

### ✅ Error Handling
- Duplicate user registration
- Invalid credentials (wrong email/password)
- Non-existent user login attempts
- Invalid/expired session tokens
- Missing required configuration
- Malformed inputs

### ✅ Security Testing
- SQL injection protection (email and password fields)
- Password hashing verification
- Session token uniqueness
- Timing attack resistance (constant-time comparisons)
- Password case sensitivity
- Secure token generation

### ✅ Edge Cases
- Empty inputs (email, password, token)
- Very long inputs (stress testing)
- Whitespace handling
- Unicode characters (passwords, emails)
- Null bytes in passwords
- Multiple dots and special chars in emails
- Concurrent operations
- Double logout (idempotency)

### ✅ Thread Safety
- `Auth` implements `Send` and `Sync`
- `AuthError` implements `Send` and `Sync`
- Concurrent login/logout operations
- Auth instance cloning

## Test Organization

```
src/tests/
├── mod.rs                    # Test module declaration
├── validation_tests.rs       # Email and password validation (13 tests)
├── integration_tests.rs      # Full auth flow testing (21 tests)
├── error_tests.rs           # Error handling and edge cases (22 tests)
└── README.md                # Detailed test documentation
```

## Test Results

```
Test Suite Statistics:
├── Total Tests: 56
├── Passed: 56
├── Failed: 0
├── Execution Time: ~5 seconds
└── Database: In-memory SQLite (isolated per test)
```

## Key Design Principles

1. **Isolation**: Each test uses a unique in-memory database
2. **No Mocking**: Real database operations for authentic testing
3. **Fast Execution**: All tests complete in under 10 seconds
4. **Deterministic**: No flaky tests, consistent results
5. **Security-First**: Explicit tests for common vulnerabilities
6. **Framework Agnostic**: Tests use the same API pattern as production

## Test Helper Functions

### `setup_test_auth()`
Creates a fresh Auth instance with in-memory SQLite database:

```rust
use crate::tests::integration_tests::setup_test_auth;

#[tokio::test]
async fn my_test() {
    let auth = setup_test_auth().await.unwrap();
    // Test code here
}
```

## Example Test Pattern

```rust
#[tokio::test]
async fn test_user_registration() {
    // Arrange
    let auth = setup_test_auth().await.unwrap();
    
    // Act
    let result = auth.register(Register {
        email: "test@example.com".into(),
        password: "SecurePass123".into(),
    }).await;
    
    // Assert
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

## Running Specific Tests

```bash
# Run validation tests only
cargo test validation_tests

# Run a specific test
cargo test test_register_user_success

# Run tests with specific string
cargo test password

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1
```

## Continuous Integration

The test suite is CI/CD ready:
- ✅ No external dependencies
- ✅ No network calls
- ✅ Deterministic behavior
- ✅ Fast execution
- ✅ Clear error messages

## Coverage Gaps (Future Work)

Areas not yet covered by tests:
- [ ] Session expiration (time-based)
- [ ] Password reset flow (when implemented)
- [ ] Email verification (when implemented)
- [ ] OAuth integration (when implemented)
- [ ] Rate limiting (when implemented)
- [ ] Audit logging (when implemented)

## Contributing Tests

When adding new features:

1. Add unit tests for validation logic
2. Add integration tests for new operations
3. Add error tests for failure scenarios
4. Consider security implications
5. Test both success and error paths
6. Use `setup_test_auth()` for consistency
7. Follow existing naming conventions

### Test Naming Convention

```rust
test_<feature>_<scenario>
├── test_register_user_success
├── test_login_wrong_password
├── test_verify_invalid_token
└── test_logout_after_expiration
```

## Debugging Failed Tests

```bash
# Show test output
cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

## Performance

All tests complete quickly:
- Average test time: ~100ms per test
- Total suite time: ~5 seconds
- In-memory database: No I/O bottlenecks
- Parallel execution: Safe and efficient

## Test Philosophy

AuthKit tests follow these principles from the project's design:

1. **Single Entry Point**: All tests use the `Auth` struct
2. **No Leaky Abstractions**: Tests don't access internals
3. **Framework Agnostic**: No web framework in tests
4. **Same API Everywhere**: Consistent with production usage
5. **Security by Default**: Tests verify secure defaults

## Related Documentation

- [Tests README](src/tests/README.md) - Detailed test documentation
- [AGENTS.md](AGENTS.md) - Project design principles
- [Cargo.toml](Cargo.toml) - Test dependencies

---

**Last Updated**: Test suite passes with 56 tests
**Database**: SQLite in-memory
**Runtime**: Tokio async runtime