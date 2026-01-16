# Rocket AuthKit Example

A complete authentication system built with [Rocket](https://rocket.rs/) and [AuthKit](https://github.com/Akshay2642005/authkit), featuring:

- ✅ User registration and login
- ✅ Session management
- ✅ Email verification with SMTP
- ✅ RESTful JSON API
- ✅ Secure password hashing (Argon2)
- ✅ Database-backed sessions (SQLite)

## Features

This example demonstrates all available AuthKit operations:

### Authentication
- **Register** - Create new user accounts with secure password hashing
- **Login** - Authenticate users and create sessions
- **Logout** - Invalidate user sessions
- **Verify Session** - Check if a session token is valid

### Email Verification
- **Send Verification** - Generate and send verification emails
- **Verify Email** - Confirm email addresses with tokens
- **Resend Verification** - Resend verification emails if needed

## Prerequisites

- Rust 1.75 or higher
- SMTP server credentials (Gmail, SendGrid, etc.)

## Quick Start

### 1. Clone and Navigate

```bash
cd authkit/examples/rocket-auth
```

### 2. Configure SMTP

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your SMTP credentials:

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@yourapp.com
APP_URL=http://localhost:8000
```

#### Gmail Setup

1. Enable 2-Factor Authentication on your Google account
2. Generate an App Password: https://myaccount.google.com/apppasswords
3. Use the generated password as `SMTP_PASSWORD`

### 3. Run the Server

```bash
cargo run
```

The server will start on `http://localhost:8000`

## API Endpoints

### General

#### Health Check
```bash
GET /health
```

Response:
```json
{
  "status": "ok",
  "service": "rocket-auth"
}
```

#### API Information
```bash
GET /
```

Returns a list of all available endpoints.

---

### Authentication

#### Register User

```bash
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

Response (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "email_verified": false,
  "created_at": 1705432800,
  "message": "User registered successfully. Please verify your email."
}
```

#### Login

```bash
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

Response (200 OK):
```json
{
  "token": "session-token-here",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "expires_at": 1705519200,
  "message": "Login successful"
}
```

#### Logout

```bash
POST /auth/logout
Content-Type: application/json

{
  "token": "session-token-here"
}
```

Response (200 OK):
```json
{
  "message": "Logout successful"
}
```

#### Verify Session

```bash
GET /auth/verify?token=session-token-here
```

Response (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "email_verified": true,
  "created_at": 1705432800
}
```

---

### Email Verification

#### Send Verification Email

```bash
POST /email/send-verification
Content-Type: application/json

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Response (200 OK):
```json
{
  "token": "verification-token-here",
  "email": "user@example.com",
  "expires_at": 1705519200,
  "message": "Verification email sent successfully"
}
```

> **Note:** An email will be automatically sent to the user with a verification link.

#### Verify Email

```bash
GET /email/verify?token=verification-token-here
```

**This endpoint can be accessed directly by clicking the link in the verification email.**

Response (200 OK):
Returns an HTML page with a success message:
- ✅ Displays a beautiful success page with the verified email
- Includes a "Go to Home" button
- Mobile-responsive design

If the verification fails, returns an HTML error page explaining the issue (expired token, already used, etc.)

> **Note:** This endpoint returns HTML, not JSON, for a better user experience when clicking email links. For API access, you can check the response status code (200 for success, 400 for failure).

#### Resend Verification Email

```bash
POST /email/resend-verification
Content-Type: application/json

{
  "email": "user@example.com"
}
```

Response (200 OK):
```json
{
  "token": "new-verification-token",
  "email": "user@example.com",
  "expires_at": 1705519200,
  "message": "Verification email resent successfully"
}
```

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error message"
}
```

### Common Error Types

- `UserNotFound` - User does not exist
- `UserAlreadyExists` - Email already registered
- `InvalidCredentials` - Wrong email or password
- `InvalidToken` - Token is invalid or malformed
- `TokenExpired` - Token has expired
- `TokenAlreadyUsed` - Token has already been used
- `SessionNotFound` - Session does not exist
- `EmailAlreadyVerified` - Email is already verified
- `ValidationError` - Input validation failed
- `EmailSendError` - Failed to send email

## Testing with cURL

### Complete Flow Example

```bash
# 1. Register a new user
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"SecurePass123!"}'

# 2. Send verification email (use user_id from registration)
curl -X POST http://localhost:8000/email/send-verification \
  -H "Content-Type: application/json" \
  -d '{"user_id":"your-user-id-here"}'

# 3. Verify email (use token from email or response)
# Note: Users typically click the link in their email, but you can also test via curl:
curl http://localhost:8000/email/verify?token=verification-token-here

# 4. Login
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"SecurePass123!"}'

# 5. Verify session (use token from login)
curl http://localhost:8000/auth/verify?token=session-token-here

# 6. Logout
curl -X POST http://localhost:8000/auth/logout \
  -H "Content-Type: application/json" \
  -d '{"token":"session-token-here"}'
```

## Project Structure

```
rocket-auth/
├── Cargo.toml              # Dependencies
├── .env.example            # Environment variables template
├── README.md               # This file
└── src/
    ├── main.rs             # Server setup and configuration
    ├── handlers.rs         # Route handlers
    ├── models.rs           # Request/response types
    └── email.rs            # SMTP email sender implementation
```

## Architecture

This example follows the **adapter pattern** recommended by AuthKit:

1. **AuthKit Core** - Handles all authentication logic
2. **Rocket Adapter** - Translates HTTP requests to AuthKit operations
3. **SMTP Email Sender** - Implements the `EmailSender` trait

```
┌─────────────────────┐
│   Rocket Routes     │  (handlers.rs)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   AuthKit Core      │  (authkit library)
└──────────┬──────────┘
           │
           ├────────► SQLite Database
           │
           └────────► SMTP Email Sender (email.rs)

                     User clicks link in email
                              │
                              ▼
                     GET /email/verify?token=xxx
                              │
                              ▼
                        HTML Success Page
```

## SMTP Provider Examples

### Gmail

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

### SendGrid

```env
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
```

### Mailgun

```env
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@your-domain.com
SMTP_PASSWORD=your-mailgun-password
```

### AWS SES

```env
SMTP_HOST=email-smtp.us-east-1.amazonaws.com
SMTP_PORT=587
SMTP_USERNAME=your-aws-ses-username
SMTP_PASSWORD=your-aws-ses-password
```

## Security Features

- ✅ **Argon2id password hashing** - Industry-standard secure hashing
- ✅ **Timing-safe comparisons** - Prevents timing attacks
- ✅ **Token expiration** - Tokens expire after 24 hours
- ✅ **Single-use tokens** - Verification tokens can only be used once
- ✅ **Session expiration** - Sessions expire after 24 hours
- ✅ **Password validation** - Enforces strong passwords
- ✅ **Email validation** - Validates email format

## Customization

### Change Database

Edit `main.rs` to use PostgreSQL instead:

```rust
let database = Database::postgres("postgresql://user:pass@localhost/authdb")
    .await
    .expect("Failed to create database");
```

### Customize Email Templates

Edit `src/email.rs` to modify the HTML email template in `build_html_body()`.

### Add Middleware

Add request guards for protected routes:

```rust
use rocket::request::{self, Request, FromRequest};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Extract token from header and verify
        // Return user if valid
    }
}

#[rocket::get("/protected")]
async fn protected(user: AuthenticatedUser) -> String {
    format!("Hello, {}!", user.email)
}
```

## Development Tips

1. **Use Console Email Sender for Testing**
   
   Replace `SmtpEmailSender` with a console sender during development:
   
   ```rust
   struct ConsoleEmailSender;
   
   #[async_trait]
   impl EmailSender for ConsoleEmailSender {
       async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
           println!("📧 Email to: {}", context.email);
           println!("🔗 Token: {}", context.token);
           Ok(())
       }
   }
   ```

2. **Enable Rocket Logging**
   
   Set `ROCKET_LOG_LEVEL=normal` for detailed logs.

3. **Database Persistence**
   
   The example uses `auth.db` for SQLite. Delete it to reset all data.

## Troubleshooting

### SMTP Authentication Failed - "The token supplied to the function is invalid"

This error typically occurs when SMTP credentials are incorrect or misconfigured:

```json
{
  "error": "EmailSendFailed",
  "message": "Email send failed: Failed to send email: Connection error: The token supplied to the function is invalid (os error -2146893048)"
}
```

**Solutions:**

#### For Gmail Users (Most Common)

1. **Use App Password (Required if you have 2FA)**
   - Gomail**: Ensure 2FA is enabled and you're using an App Password
- **Other providers**: Check that your credentials are correct
- **Firewall**: Ensure port 587 (or your SMTP port) is not blocked

### Email Not Received

- Check spam/junk folder
- Verify SMTP credentials are correct
- Check server logs for error messages
- Test SMTP connection with a tool like `telnet`

### Database Locked

- SQLite only allows one writer at a time
- For production, use PostgreSQL
- Close other connections to `auth.db`

### Port Already in Use

Change the port in `Rocket.toml`:

```toml
[default]
port = 8080
```

## License

This example is part of the AuthKit project and follows the same license terms (MIT OR Apache-2.0).

## Links

- **AuthKit Repository**: https://github.com/Akshay2642005/authkit
- **Rocket Documentation**: https://rocket.rs/
- **Lettre Documentation**: https://lettre.rs/

## Support

For issues or questions:
- Open an issue on the AuthKit repository
- Check the AuthKit documentation
- Review the example code in this directory

---

**Built with ❤️ using AuthKit and Rocket**