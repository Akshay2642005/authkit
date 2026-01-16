# Email Integration Guide

This guide explains how to integrate email sending into AuthKit for verification emails, password resets, and other email-based authentication flows.

## Overview

AuthKit provides a flexible email integration system that allows you to:

- **Use any email service** (SendGrid, AWS SES, SMTP, Resend, etc.)
- **Choose your own templates** and email design
- **Work in development** without sending real emails
- **Maintain framework-agnostic** design

## How It Works

AuthKit uses the **Strategy Pattern** for email sending:

1. You implement the `EmailSender` trait with your email service logic
2. You pass your implementation to the `Auth` builder
3. AuthKit calls your implementation automatically when sending verification emails

If no `EmailSender` is configured, AuthKit still generates tokens but doesn't send emails automatically. This gives you full control.

## Quick Start

### 1. Basic Setup (Manual Email Handling)

By default, AuthKit generates tokens but doesn't send emails:

```rust
use authkit::prelude::*;

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;

// Register user
let user = auth.register(Register {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;

// Generate token (no email sent)
let verification = auth.send_email_verification(SendEmailVerification {
    user_id: user.id.clone(),
}).await?;

// You handle sending the email
your_email_service::send(
    &verification.email,
    &verification.token,
).await?;
```

### 2. Automatic Email Sending

Configure an `EmailSender` to send emails automatically:

```rust
use authkit::prelude::*;
use authkit::email::{EmailSender, EmailContext};
use async_trait::async_trait;

// Implement the EmailSender trait
struct MyEmailSender {
    api_key: String,
}

#[async_trait]
impl EmailSender for MyEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        // context.email - recipient's email
        // context.token - verification token (plaintext)
        // context.expires_at - expiration timestamp
        
        let url = format!("https://myapp.com/verify?token={}", context.token);
        
        // Use your email service here
        sendgrid::send(
            &self.api_key,
            &context.email,
            "Verify your email",
            &format!("Click here: {}", url),
        ).await?;
        
        Ok(())
    }
}

// Configure Auth with your email sender
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .email_sender(Box::new(MyEmailSender {
        api_key: "your_api_key".to_string(),
    }))
    .build()?;

// Now emails are sent automatically
let user = auth.register(Register {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;

// This will generate the token AND send the email
auth.send_email_verification(SendEmailVerification {
    user_id: user.id,
}).await?;
```

## The EmailSender Trait

```rust
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()>;
}

pub struct EmailContext {
    pub email: String,      // Recipient's email
    pub token: String,      // Verification token (plaintext)
    pub expires_at: i64,    // Unix timestamp
}
```

## Implementation Examples

### Example 1: Console Logger (Development)

Perfect for local development and testing:

```rust
use authkit::email::{EmailSender, EmailContext};
use authkit::Result;
use async_trait::async_trait;

struct ConsoleEmailSender {
    base_url: String,
}

#[async_trait]
impl EmailSender for ConsoleEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        println!("📧 ============= EMAIL =============");
        println!("To: {}", context.email);
        println!("Verify at: {}/verify?token={}", self.base_url, context.token);
        println!("===================================");
        Ok(())
    }
}

// Usage
let auth = Auth::builder()
    .database(Database::sqlite(":memory:").await?)
    .email_sender(Box::new(ConsoleEmailSender {
        base_url: "http://localhost:3000".to_string(),
    }))
    .build()?;
```

### Example 2: SendGrid

Production-ready with SendGrid:

```rust
use authkit::email::{EmailSender, EmailContext};
use authkit::Result;
use async_trait::async_trait;

struct SendGridEmailSender {
    api_key: String,
    from_email: String,
    base_url: String,
}

#[async_trait]
impl EmailSender for SendGridEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        let verification_url = format!("{}/verify?token={}", self.base_url, context.token);
        
        // Use sendgrid crate or HTTP client
        let client = reqwest::Client::new();
        
        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "personalizations": [{
                    "to": [{"email": context.email}]
                }],
                "from": {"email": self.from_email},
                "subject": "Verify your email address",
                "content": [{
                    "type": "text/html",
                    "value": format!(
                        r#"
                        <html>
                            <body>
                                <h1>Verify Your Email</h1>
                                <p>Click the link below to verify your email address:</p>
                                <a href="{}" style="background: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">
                                    Verify Email
                                </a>
                                <p>Or copy this link: {}</p>
                            </body>
                        </html>
                        "#,
                        verification_url, verification_url
                    )
                }]
            }))
            .send()
            .await
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(authkit::AuthError::EmailSendFailed(
                format!("SendGrid API error: {}", response.status())
            ));
        }
        
        Ok(())
    }
}

// Usage
let auth = Auth::builder()
    .database(Database::postgres(&db_url).await?)
    .email_sender(Box::new(SendGridEmailSender {
        api_key: std::env::var("SENDGRID_API_KEY")?,
        from_email: "noreply@myapp.com".to_string(),
        base_url: "https://myapp.com".to_string(),
    }))
    .build()?;
```

### Example 3: AWS SES

Using AWS Simple Email Service:

```rust
use authkit::email::{EmailSender, EmailContext};
use authkit::Result;
use async_trait::async_trait;
use aws_sdk_sesv2::{Client, types::{Body, Content, Destination, EmailContent, Message}};

struct AwsSesEmailSender {
    client: Client,
    from_email: String,
    base_url: String,
}

#[async_trait]
impl EmailSender for AwsSesEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        let verification_url = format!("{}/verify?token={}", self.base_url, context.token);
        
        let dest = Destination::builder()
            .to_addresses(&context.email)
            .build();
        
        let subject = Content::builder()
            .data("Verify your email address")
            .build()
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        let body_html = Content::builder()
            .data(format!(
                r#"<html><body>
                <h1>Verify Your Email</h1>
                <p>Click here: <a href="{}">Verify Email</a></p>
                </body></html>"#,
                verification_url
            ))
            .build()
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        let body = Body::builder()
            .html(body_html)
            .build();
        
        let message = Message::builder()
            .subject(subject)
            .body(body)
            .build();
        
        let email_content = EmailContent::builder()
            .simple(message)
            .build();
        
        self.client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(dest)
            .content(email_content)
            .send()
            .await
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        Ok(())
    }
}
```

### Example 4: SMTP (Self-Hosted)

Using the `lettre` crate for SMTP:

```rust
use authkit::email::{EmailSender, EmailContext};
use authkit::Result;
use async_trait::async_trait;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

struct SmtpEmailSender {
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    base_url: String,
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        let verification_url = format!("{}/verify?token={}", self.base_url, context.token);
        
        let email = Message::builder()
            .from(self.from_email.parse().unwrap())
            .to(context.email.parse().unwrap())
            .subject("Verify your email address")
            .body(format!(
                "Click here to verify your email: {}",
                verification_url
            ))
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        let creds = Credentials::new(
            self.smtp_username.clone(),
            self.smtp_password.clone(),
        );
        
        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?
            .credentials(creds)
            .build();
        
        mailer
            .send(&email)
            .map_err(|e| authkit::AuthError::EmailSendFailed(e.to_string()))?;
        
        Ok(())
    }
}
```

## Best Practices

### 1. Environment-Based Configuration

Use different email senders for different environments:

```rust
fn create_email_sender(env: &str) -> Box<dyn EmailSender> {
    match env {
        "production" => Box::new(SendGridEmailSender {
            api_key: std::env::var("SENDGRID_API_KEY").unwrap(),
            from_email: "noreply@myapp.com".into(),
            base_url: "https://myapp.com".into(),
        }),
        "development" => Box::new(ConsoleEmailSender {
            base_url: "http://localhost:3000".into(),
        }),
        _ => panic!("Unknown environment"),
    }
}

let auth = Auth::builder()
    .database(database)
    .email_sender(create_email_sender(&env))
    .build()?;
```

### 2. HTML Email Templates

Create reusable HTML templates:

```rust
fn verification_email_html(url: &str, email: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
        </head>
        <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: #f5f5f5; padding: 20px;">
                <h1 style="color: #333;">Welcome to MyApp!</h1>
                <p>Hi {},</p>
                <p>Thanks for signing up! Please verify your email address by clicking the button below:</p>
                <a href="{}" style="display: inline-block; background: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; margin: 20px 0;">
                    Verify Email Address
                </a>
                <p style="color: #666; font-size: 12px;">
                    Or copy and paste this link into your browser:<br>
                    <a href="{}">{}</a>
                </p>
                <p style="color: #666; font-size: 12px;">
                    This link will expire in 24 hours.
                </p>
            </div>
        </body>
        </html>
        "#,
        email, url, url, url
    )
}
```

### 3. Error Handling

Always handle email sending errors gracefully:

```rust
#[async_trait]
impl EmailSender for MyEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        match self.send_internal(&context).await {
            Ok(_) => {
                log::info!("Verification email sent to {}", context.email);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to send email to {}: {}", context.email, e);
                Err(authkit::AuthError::EmailSendFailed(e.to_string()))
            }
        }
    }
}
```

### 4. Rate Limiting

Consider adding rate limiting to prevent abuse:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

struct RateLimitedEmailSender {
    inner: Box<dyn EmailSender>,
    last_sent: Arc<Mutex<HashMap<String, SystemTime>>>,
    min_interval: Duration,
}

#[async_trait]
impl EmailSender for RateLimitedEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        let mut last_sent = self.last_sent.lock().unwrap();
        
        if let Some(last_time) = last_sent.get(&context.email) {
            if last_time.elapsed().unwrap() < self.min_interval {
                return Err(authkit::AuthError::RateLimitExceeded(
                    "Please wait before requesting another email".into()
                ));
            }
        }
        
        self.inner.send_verification_email(context.clone()).await?;
        last_sent.insert(context.email, SystemTime::now());
        
        Ok(())
    }
}
```

## Integration with Web Frameworks

### Axum Example

```rust
use axum::{extract::State, Json, http::StatusCode};
use authkit::prelude::*;

#[derive(Clone)]
struct AppState {
    auth: Auth,
}

async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<StatusCode, AppError> {
    // Register user
    let user = state.auth.register(Register {
        email: req.email.clone(),
        password: req.password,
    }).await?;
    
    // Send verification email (automatic if EmailSender configured)
    state.auth.send_email_verification(SendEmailVerification {
        user_id: user.id,
    }).await?;
    
    Ok(StatusCode::CREATED)
}
```

## Testing

### Mock Email Sender for Tests

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct MockEmailSender {
    sent_emails: Arc<Mutex<Vec<EmailContext>>>,
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
        self.sent_emails.lock().unwrap().push(context);
        Ok(())
    }
}

#[tokio::test]
async fn test_email_verification() {
    let mock_sender = MockEmailSender::default();
    let sent_emails = mock_sender.sent_emails.clone();
    
    let auth = Auth::builder()
        .database(Database::sqlite(":memory:").await.unwrap())
        .email_sender(Box::new(mock_sender))
        .build()
        .unwrap();
    
    auth.migrate().await.unwrap();
    
    let user = auth.register(Register {
        email: "test@example.com".into(),
        password: "password".into(),
    }).await.unwrap();
    
    auth.send_email_verification(SendEmailVerification {
        user_id: user.id,
    }).await.unwrap();
    
    // Verify email was "sent"
    let emails = sent_emails.lock().unwrap();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].email, "test@example.com");
}
```

## Summary

**Key Points:**

1. **Optional** - Email sending is opt-in via the builder
2. **Flexible** - Use any email service you prefer
3. **Testable** - Easy to mock for testing
4. **Framework-agnostic** - Works anywhere AuthKit works
5. **Simple** - One trait, one method to implement

**When emails are sent automatically:**

- `send_email_verification()` - Sends verification email
- `resend_email_verification()` - Sends new verification email

**When emails are NOT sent (no EmailSender configured):**

- You receive the token and handle email sending manually
- Perfect for custom flows or non-email delivery methods

For more examples, see `examples/email_sender.rs` in the AuthKit repository.