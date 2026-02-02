//! Rocket Authentication Example with SMTP Email Sending
//!
//! This example demonstrates a complete authentication system using Rocket and AuthKit
//! with all available routes and SMTP email verification.
//!
//! Features:
//! - User registration with email verification
//! - Login/logout with session management
//! - Email verification flow (send, verify, resend)
//! - SMTP email sending using lettre
//! - RESTful JSON API
//!
//! Run with: cargo run
//! Configure SMTP in .env or environment variables

mod email;
mod handlers;
mod models;

use authkit::prelude::*;
use email::SmtpEmailSender;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::{launch, routes};
use sqlx::Executor;

/// Application state containing the Auth instance
pub struct AppState {
  auth: Auth,
}

/// Health check endpoint
#[rocket::get("/health")]
async fn health() -> Json<serde_json::Value> {
  Json(serde_json::json!({
      "status": "ok",
      "service": "rocket-auth"
  }))
}

/// Root endpoint with API information
#[rocket::get("/")]
async fn index() -> Json<serde_json::Value> {
  Json(serde_json::json!({
      "service": "Rocket AuthKit Example",
      "version": "0.1.0",
      "endpoints": {
          "auth": {
              "register": "POST /auth/register",
              "login": "POST /auth/login",
              "logout": "POST /auth/logout",
              "verify_session": "GET /auth/verify",
          },
          "email": {
              "send_verification": "POST /email/send-verification",
              "verify_email": "GET /email/verify?token=<token> (HTML response, clickable from email)",
              "resend_verification": "POST /email/resend-verification",
          },
          "health": "GET /health"
      }
  }))
}

/// Error catcher for 404 Not Found
#[rocket::catch(404)]
fn not_found() -> Custom<Json<models::ErrorResponse>> {
  Custom(
    Status::NotFound,
    Json(models::ErrorResponse {
      error: "Not Found".to_string(),
      message: "The requested resource was not found".to_string(),
    }),
  )
}

/// Error catcher for 500 Internal Server Error
#[rocket::catch(500)]
fn internal_error() -> Custom<Json<models::ErrorResponse>> {
  Custom(
    Status::InternalServerError,
    Json(models::ErrorResponse {
      error: "Internal Server Error".to_string(),
      message: "An internal server error occurred".to_string(),
    }),
  )
}

#[launch]
async fn rocket() -> _ {
  // Load environment variables from .env file if it exists
  dotenv::dotenv().ok();

  println!("🚀 Starting Rocket AuthKit Example Server...\n");

  // SMTP Configuration
  let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
  let smtp_port = std::env::var("SMTP_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(587);
  let smtp_username =
    std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "your-email@gmail.com".to_string());
  let smtp_password =
    std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "your-app-password".to_string());
  let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| smtp_username.clone());
  let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

  println!("📧 SMTP Configuration:");
  println!("   Host: {}:{}", smtp_host, smtp_port);
  println!("   From: {}", smtp_from);
  println!("   App URL: {}\n", app_url);

  // Create SMTP email sender
  let email_sender = SmtpEmailSender::new(
    smtp_host,
    smtp_port,
    smtp_username,
    smtp_password,
    smtp_from,
    app_url,
  );

  // Initialize database (SQLite for this example)
  let database = Database::sqlite("auth.db")
    .await
    .expect("Failed to create database");

  println!("💾 Database: auth.db (SQLite)\n");

  // Build Auth instance with email sender
  let auth = Auth::builder()
    .database(database)
    .email_sender(Box::new(email_sender))
    .build()
    .expect("Failed to build Auth");

  // Note: In production, run migrations via CLI before starting the server:
  //   authkit migrate --db-url sqlite://auth.db
  // For this demo, we set up the schema inline
  println!("📦 Setting up database schema...");
  setup_demo_schema(&database)
    .await
    .expect("Failed to set up schema");
  println!("✅ Schema setup complete\n");

  println!("🔐 AuthKit initialized successfully");
  println!("🌐 Server starting on http://localhost:8000\n");
  println!("📚 API Documentation:");
  println!("   GET  /                              - API information");
  println!("   GET  /health                        - Health check");
  println!("   POST /auth/register                 - Register new user");
  println!("   POST /auth/login                    - Login user");
  println!("   POST /auth/logout                   - Logout user");
  println!("   GET  /auth/verify?token=<TOKEN>     - Verify session");
  println!("   POST /email/send-verification       - Send verification email");
  println!("   POST /email/verify                  - Verify email with token");
  println!("   POST /email/resend-verification     - Resend verification email");
  println!("\n💡 Tips:");
  println!("   - Configure SMTP in .env file or environment variables");
  println!("   - Use SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD");
  println!("   - Set APP_URL for email verification links");
  println!("\n");

  // Create app state
  let state = AppState { auth };

  rocket::build()
    .manage(state)
    .mount(
      "/",
      routes![
        index,
        health,
        handlers::register,
        handlers::login,
        handlers::logout,
        handlers::verify_session,
        handlers::send_verification,
        handlers::verify_email,
        handlers::resend_verification,
      ],
    )
    .register("/", rocket::catchers![not_found, internal_error])
}

/// Set up database schema for this demo
/// In production, use the CLI: `authkit migrate --db-url sqlite://auth.db`
async fn setup_demo_schema(db: &Database) -> std::result::Result<(), Box<dyn std::error::Error>> {
  let pool = match &db.inner {
    authkit::types::DatabaseInner::Sqlite(sqlite_db) => &sqlite_db.pool,
    #[allow(unreachable_patterns)]
    _ => panic!("This example requires SQLite"),
  };

  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        name TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        email_verified INTEGER NOT NULL DEFAULT 0,
        email_verified_at INTEGER
      )
      "#,
    )
    .await?;

  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS accounts (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        provider TEXT NOT NULL,
        provider_account_id TEXT NOT NULL,
        password_hash TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(provider, provider_account_id)
      )
      "#,
    )
    .await?;

  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        token TEXT NOT NULL UNIQUE,
        expires_at INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        ip_address TEXT,
        user_agent TEXT
      )
      "#,
    )
    .await?;

  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS verification (
        id TEXT PRIMARY KEY,
        user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
        identifier TEXT NOT NULL,
        token_hash TEXT NOT NULL UNIQUE,
        token_type TEXT NOT NULL,
        expires_at INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        used_at INTEGER
      )
      "#,
    )
    .await?;

  Ok(())
}
