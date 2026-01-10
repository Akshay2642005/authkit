# AuthKit Test Suite

This directory contains comprehensive tests for the AuthKit authentication library.

## Test Organization

The test suite is organized into three main modules:

### 1. `validation_tests.rs`
Unit tests for input validation logic.

**Coverage:**
- Email validation (valid/invalid formats)
- Password strength validation
- Password requirements:
  - Minimum length (8 characters)
  - Maximum length (128 characters)
  - Uppercase requirement
  - Lowercase requirement
  - Digit requirement
- Edge cases (empty inputs, boundary values, unicode)

**Example tests:**
- `test_valid_email` - Valid email formats
- `test_invalid_email` - Invalid email formats
- `test_password_too_short` - Password length validation
- `test_password_no_uppercase` - Missing uppercase letter

### 2. `integration_tests.rs`
End-to-end integration tests for the complete authentication flow.

**Coverage:**
- User registration
- Login and session creation
- Session verification
- Logout and session invalidation
- Multiple concurrent sessions
- Auth instance cloning
- Full lifecycle testing

**Example tests:**
- `test_register_user_success` - Successful user registration
- `test_login_success` - User login flow
- `test_verify_session_success` - Session verification
- `test_logout_success` - Logout flow
- `test_full_auth_lifecycle` - Complete register → login → verify → logout

**Test Helper:**
- `setup_test_auth()` - Creates an isolated Auth instance with in-memory SQLite database

### 3. `error_tests.rs`
Error handling, edge cases, and security tests.

**Coverage:**
- Error conditions (missing config, invalid credentials)
- Edge cases (empty inputs, very long inputs)
- Security testing (SQL injection attempts, timing attacks)
- Concurrency handling
- Unicode and special character handling
- Thread safety verification (Send/Sync traits)

**Example tests:**
- `test_builder_missing_database` - Builder validation
- `test_sql_injection_in_email` - SQL injection protection
- `test_concurrent_operations` - Concurrent access
- `test_double_logout` - Idempotent operations

## Running Tests

### Run all tests
```bash
cargo test
```

### Run only library tests
```bash
cargo test --lib
```

### Run a specific test module
```bash
cargo test validation_tests
cargo test integration_tests
cargo test error_tests
```

### Run a specific test
```bash
cargo test test_register_user_success
```

### Run with output
```bash
cargo test -- --nocapture
```

### Run with verbose output
```bash
cargo test -- --nocapture --test-threads=1
```

## Test Statistics

**Total Tests:** 56
- Validation tests: 13
- Integration tests: 21
- Error/edge case tests: 22

**Code Coverage Areas:**
- ✅ Email validation
- ✅ Password validation
- ✅ User registration
- ✅ User login
- ✅ Session verification
- ✅ Session logout
- ✅ Duplicate user handling
- ✅ Invalid credentials
- ✅ Session expiration
- ✅ Multiple sessions per user
- ✅ Concurrent operations
- ✅ SQL injection protection
- ✅ Unicode handling
- ✅ Thread safety

## Test Principles

1. **Isolation**: Each test uses an independent in-memory database
2. **No Mocking**: Real database operations for authentic integration testing
3. **Fast**: In-memory SQLite ensures sub-second test execution
4. **Deterministic**: Tests produce consistent results
5. **Security-focused**: Explicit tests for common vulnerabilities

## Adding New Tests

When adding new features to AuthKit, follow these guidelines:

1. **Add validation tests** for any new input validation logic
2. **Add integration tests** for new operations or workflows
3. **Add error tests** for new error conditions or edge cases
4. **Use `setup_test_auth()`** helper for consistency
5. **Test both success and failure paths**
6. **Consider security implications** (injection, timing attacks, etc.)

### Example Test Template

```rust
#[tokio::test]
async fn test_your_new_feature() {
    // Setup
    let auth = setup_test_auth().await.unwrap();
    
    // Execute
    let result = auth.your_operation(/* params */).await;
    
    // Assert
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.field, expected_value);
}
```

## Test Data Conventions

- **Email format**: `{purpose}@example.com`
  - Example: `test@example.com`, `duplicate@example.com`
- **Password format**: Use "SecurePass123" for valid passwords
- **Token handling**: Test with both valid and invalid tokens

## Continuous Integration

These tests are designed to run in CI/CD pipelines:
- No external dependencies required
- Fast execution (< 15 seconds for full suite)
- No flaky tests
- Clear failure messages

## Design Philosophy

The AuthKit test suite follows the project's core principles:

1. **Single Entry Point**: All tests interact with the `Auth` struct
2. **Framework Agnostic**: No web framework dependencies
3. **No Leaky Abstractions**: Tests don't access internal implementation
4. **Security by Default**: Explicit tests for secure behavior
5. **Same API Everywhere**: Tests verify consistent behavior across contexts

## Troubleshooting

### Tests fail with database errors
- Ensure SQLite feature is enabled in `Cargo.toml`
- Check that migrations are running (`auth.migrate().await`)

### Tests hang or timeout
- Verify async runtime is configured correctly
- Check for deadlocks in concurrent tests

### Sporadic test failures
- Tests should be deterministic; investigate race conditions
- Ensure proper test isolation (each test should be independent)