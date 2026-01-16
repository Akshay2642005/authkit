# 🎉 Announcing AuthKit v0.1.0 - Authentication Made Simple for Rust

I'm excited to announce the first public release of **AuthKit**, a better-auth inspired authentication library for Rust!

## What is AuthKit?

AuthKit is a **framework-agnostic authentication library** that provides secure, database-backed user authentication with a clean, simple API. Think of it as bringing the developer experience of [better-auth](https://github.com/better-auth/better-auth) (TypeScript) to the Rust ecosystem.

## Key Features

✨ **Simple API** - One `Auth` object, no complexity
```rust
let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;
```

🔐 **Secure by Default**
- Argon2id password hashing
- Cryptographically secure tokens
- SQL injection protection
- Timing-safe comparisons

📧 **Email Verification**
- Generate verification tokens
- Single-use, time-limited tokens
- Resend functionality
```rust
let verification = auth.send_email_verification(SendEmailVerification {
    user_id: user.id,
}).await?;

auth.verify_email(VerifyEmail {
    token: verification.token,
}).await?;
```

💾 **Multi-Database Support**
- SQLite (perfect for development)
- PostgreSQL (production-ready)
- More coming soon

🚀 **Framework-Agnostic**
- Works with Axum, Actix, Rocket, or standalone
- Use in web servers, CLI tools, workers, anywhere
- No framework lock-in

## Quick Example

```rust
use authkit::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let auth = Auth::builder()
        .database(Database::sqlite("auth.db").await?)
        .build()?;
    
    auth.migrate().await?;
    
    // Register
    let user = auth.register(Register {
        email: "user@example.com".into(),
        password: "SecurePass123!".into(),
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
    
    Ok(())
}
```

## Why AuthKit?

### For Application Developers
- **Less Code** - Focus on your app, not auth infrastructure
- **Production Ready** - 68 tests, comprehensive error handling
- **Flexible** - Choose your database, hashing algorithm, and features
- **Well Documented** - Complete API docs and working examples

### For Library Authors
- **No Framework Coupling** - Integrate with any web framework
- **Extensible** - Strategy pattern for customization
- **Clean Boundaries** - No leaked abstractions

### Design Philosophy
1. **Single Entry Point** - Everything through `Auth` object
2. **Framework-Agnostic** - Works everywhere Rust works
3. **Opinionated Defaults** - Secure by default
4. **No Leaky Abstractions** - Clean public API
5. **Same API Everywhere** - Consistent experience

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
authkit = "0.1"
tokio = { version = "1.28", features = ["full"] }
```

## What's Included in v0.1.0

✅ User registration and login  
✅ Session management  
✅ Email verification (send, verify, resend)  
✅ SQLite and PostgreSQL support  
✅ Argon2id and bcrypt password hashing  
✅ Automatic migrations  
✅ Comprehensive test suite (68 tests)  
✅ Complete documentation  

## Coming Soon

**v0.2.0** (Next)
- Password reset flow
- Magic link authentication
- JWT sessions
- Refresh tokens

**v0.3.0**
- OAuth/SSO integration
- Two-factor authentication
- Framework adapters

## Try It Out

```bash
# Clone the repository
git clone https://github.com/Akshay2642005/authkit

# Run the example
cd authkit
cargo run --example email_verification --features sqlite
```

## Links

- **GitHub**: https://github.com/Akshay2642005/authkit
- **Crates.io**: https://crates.io/crates/authkit (after release)
- **Documentation**: https://docs.rs/authkit (after release)
- **Examples**: https://github.com/Akshay2642005/authkit/tree/main/examples

## License

Dual-licensed under MIT or Apache-2.0, your choice.

## Feedback Welcome!

This is the first public release, and I'd love to hear your feedback:
- What features would you like to see next?
- How can AuthKit better serve your use cases?
- Found a bug? Open an issue!

## Special Thanks

- Inspired by [better-auth](https://github.com/better-auth/better-auth)
- Built on amazing Rust crates: Tokio, SQLx, Argon2
- Thanks to the Rust community for excellent tools and support

---

**AuthKit aims to make authentication in Rust as simple and secure as possible. Give it a try and let me know what you think!**

Happy coding! 🦀