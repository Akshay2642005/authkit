//! Route Handlers for Rocket Authentication API
//!
//! This module contains all the HTTP route handlers that translate
//! Rocket requests into AuthKit operations and format responses.

use crate::models::*;
use crate::AppState;
use authkit::prelude::*;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;

// ============================================================================
// Authentication Handlers
// ============================================================================

/// POST /auth/register - Register a new user
///
/// Request body:
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "secure-password"
/// }
/// ```
#[rocket::post("/auth/register", data = "<request>")]
pub async fn register(
  state: &State<AppState>,
  request: Json<RegisterRequest>,
) -> std::result::Result<Json<RegisterResponse>, Custom<Json<ErrorResponse>>> {
  let result = state
    .auth
    .register(Register {
      email: request.email.clone(),
      password: request.password.clone(),
    })
    .await;

  match result {
    Ok(user) => {
      // Automatically send verification email
      let user_id = user.id.clone();
      let email = user.email.clone();

      let verification_result = state
        .auth
        .send_email_verification(SendEmailVerification {
          user_id: user_id.clone(),
        })
        .await;

      let message = match verification_result {
        Ok(_) => format!("User registered successfully. A verification email has been sent to {}.", email),
        Err(_) => "User registered successfully. Failed to send verification email, but you can request a new one.".to_string(),
      };

      Ok(Json(RegisterResponse {
        id: user_id,
        email: email,
        email_verified: user.email_verified,
        created_at: user.created_at,
        message,
      }))
    }
    Err(e) => Err(Custom(
      Status::BadRequest,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}

/// POST /auth/login - Login a user
///
/// Request body:
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "secure-password"
/// }
/// ```
#[rocket::post("/auth/login", data = "<request>")]
pub async fn login(
  state: &State<AppState>,
  request: Json<LoginRequest>,
) -> std::result::Result<Json<LoginResponse>, Custom<Json<ErrorResponse>>> {
  let result = state
    .auth
    .login(Login {
      email: request.email.clone(),
      password: request.password.clone(),
    })
    .await;

  match result {
    Ok(session) => Ok(Json(LoginResponse {
      token: session.token,
      user_id: session.user_id,
      expires_at: session.expires_at,
      message: "Login successful".to_string(),
    })),
    Err(e) => Err(Custom(
      Status::Unauthorized,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}

/// POST /auth/logout - Logout a user
///
/// Request body:
/// ```json
/// {
///   "token": "session-token"
/// }
/// ```
#[rocket::post("/auth/logout", data = "<request>")]
pub async fn logout(
  state: &State<AppState>,
  request: Json<LogoutRequest>,
) -> std::result::Result<Json<LogoutResponse>, Custom<Json<ErrorResponse>>> {
  let result = state
    .auth
    .logout(Logout {
      token: request.token.clone(),
    })
    .await;

  match result {
    Ok(_) => Ok(Json(LogoutResponse {
      message: "Logout successful".to_string(),
    })),
    Err(e) => Err(Custom(
      Status::BadRequest,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}

/// GET /auth/verify?token=<token> - Verify a session token
///
/// Query parameter: token
#[rocket::get("/auth/verify?<token>")]
pub async fn verify_session(
  state: &State<AppState>,
  token: String,
) -> std::result::Result<Json<VerifyResponse>, Custom<Json<ErrorResponse>>> {
  let result = state.auth.verify(Verify { token }).await;

  match result {
    Ok(user) => Ok(Json(VerifyResponse {
      id: user.id,
      email: user.email,
      email_verified: user.email_verified,
      created_at: user.created_at,
    })),
    Err(e) => Err(Custom(
      Status::Unauthorized,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}

// ============================================================================
// Email Verification Handlers
// ============================================================================

/// POST /email/send-verification - Send email verification
///
/// Request body:
/// ```json
/// {
///   "user_id": "user-uuid"
/// }
/// ```
#[rocket::post("/email/send-verification", data = "<request>")]
pub async fn send_verification(
  state: &State<AppState>,
  request: Json<SendVerificationRequest>,
) -> std::result::Result<Json<SendVerificationResponse>, Custom<Json<ErrorResponse>>> {
  let result = state
    .auth
    .send_email_verification(SendEmailVerification {
      user_id: request.user_id.clone(),
    })
    .await;

  match result {
    Ok(verification) => Ok(Json(SendVerificationResponse {
      token: verification.token,
      email: verification.email,
      expires_at: verification.expires_at,
      message: "Verification email sent successfully".to_string(),
    })),
    Err(e) => Err(Custom(
      Status::BadRequest,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}

/// GET /email/verify?token=<token> - Verify email with token
///
/// Query parameter: token
#[rocket::get("/email/verify?<token>")]
pub async fn verify_email(
  state: &State<AppState>,
  token: String,
) -> std::result::Result<RawHtml<String>, RawHtml<String>> {
  let result = state
    .auth
    .verify_email(VerifyEmail {
      token: token.clone(),
    })
    .await;

  match result {
    Ok(user) => Ok(RawHtml(format!(
      r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Email Verified Successfully</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }}
        .container {{
            background: white;
            border-radius: 16px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            max-width: 500px;
            width: 100%;
            padding: 40px;
            text-align: center;
            animation: slideIn 0.5s ease-out;
        }}
        @keyframes slideIn {{
            from {{
                opacity: 0;
                transform: translateY(-30px);
            }}
            to {{
                opacity: 1;
                transform: translateY(0);
            }}
        }}
        .icon {{
            width: 80px;
            height: 80px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 24px;
            animation: scaleIn 0.6s ease-out 0.2s backwards;
        }}
        @keyframes scaleIn {{
            from {{
                transform: scale(0);
            }}
            to {{
                transform: scale(1);
            }}
        }}
        .icon svg {{
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
            stroke-linecap: round;
            stroke-linejoin: round;
        }}
        h1 {{
            color: #2d3748;
            font-size: 28px;
            margin-bottom: 16px;
            font-weight: 700;
        }}
        p {{
            color: #718096;
            font-size: 16px;
            line-height: 1.6;
            margin-bottom: 24px;
        }}
        .email {{
            background: #f7fafc;
            padding: 12px 20px;
            border-radius: 8px;
            color: #4a5568;
            font-weight: 600;
            margin-bottom: 24px;
            display: inline-block;
        }}
        .button {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 14px 32px;
            border-radius: 8px;
            text-decoration: none;
            display: inline-block;
            font-weight: 600;
            font-size: 16px;
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        .button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 10px 25px rgba(102, 126, 234, 0.4);
        }}
        .footer {{
            margin-top: 32px;
            padding-top: 24px;
            border-top: 1px solid #e2e8f0;
            color: #a0aec0;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
        </div>
        <h1>✅ Email Verified!</h1>
        <p>Your email address has been successfully verified.</p>
        <div class="email">{}</div>
        <p>You can now close this window and log in to your account.</p>
        <a href="/" class="button">Go to Home</a>
        <div class="footer">
            <p>Thank you for verifying your email address.</p>
        </div>
    </div>
</body>
</html>
"#,
      user.email
    ))),
    Err(e) => Err(RawHtml(format!(
      r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verification Failed</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }}
        .container {{
            background: white;
            border-radius: 16px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            max-width: 500px;
            width: 100%;
            padding: 40px;
            text-align: center;
            animation: slideIn 0.5s ease-out;
        }}
        @keyframes slideIn {{
            from {{
                opacity: 0;
                transform: translateY(-30px);
            }}
            to {{
                opacity: 1;
                transform: translateY(0);
            }}
        }}
        .icon {{
            width: 80px;
            height: 80px;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 24px;
            animation: scaleIn 0.6s ease-out 0.2s backwards;
        }}
        @keyframes scaleIn {{
            from {{
                transform: scale(0);
            }}
            to {{
                transform: scale(1);
            }}
        }}
        .icon svg {{
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
            stroke-linecap: round;
            stroke-linejoin: round;
        }}
        h1 {{
            color: #2d3748;
            font-size: 28px;
            margin-bottom: 16px;
            font-weight: 700;
        }}
        p {{
            color: #718096;
            font-size: 16px;
            line-height: 1.6;
            margin-bottom: 24px;
        }}
        .error-box {{
            background: #fff5f5;
            border: 1px solid #feb2b2;
            padding: 16px;
            border-radius: 8px;
            color: #c53030;
            margin-bottom: 24px;
        }}
        .button {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 14px 32px;
            border-radius: 8px;
            text-decoration: none;
            display: inline-block;
            font-weight: 600;
            font-size: 16px;
            transition: transform 0.2s, box-shadow 0.2s;
            margin-right: 12px;
        }}
        .button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 10px 25px rgba(102, 126, 234, 0.4);
        }}
        .button-secondary {{
            background: #e2e8f0;
            color: #2d3748;
        }}
        .button-secondary:hover {{
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
        }}
        .footer {{
            margin-top: 32px;
            padding-top: 24px;
            border-top: 1px solid #e2e8f0;
            color: #a0aec0;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
        </div>
        <h1>❌ Verification Failed</h1>
        <p>We couldn't verify your email address.</p>
        <div class="error-box">
            <strong>Error:</strong> {}
        </div>
        <p>This could happen if:</p>
        <ul style="text-align: left; color: #718096; margin: 0 auto 24px; max-width: 320px;">
            <li>The verification link has expired</li>
            <li>The link has already been used</li>
            <li>The link is invalid or corrupted</li>
        </ul>
        <div>
            <a href="/" class="button">Go to Home</a>
            <a href="/email/resend-verification" class="button button-secondary">Resend Email</a>
        </div>
        <div class="footer">
            <p>Need help? Contact support.</p>
        </div>
    </div>
</body>
</html>
"#,
      e
    ))),
  }
}

/// POST /email/resend-verification - Resend verification email
///
/// Request body:
/// ```json
/// {
///   "email": "user@example.com"
/// }
/// ```
#[rocket::post("/email/resend-verification", data = "<request>")]
pub async fn resend_verification(
  state: &State<AppState>,
  request: Json<ResendVerificationRequest>,
) -> std::result::Result<Json<ResendVerificationResponse>, Custom<Json<ErrorResponse>>> {
  let result = state
    .auth
    .resend_email_verification(ResendEmailVerification {
      email: request.email.clone(),
    })
    .await;

  match result {
    Ok(verification) => Ok(Json(ResendVerificationResponse {
      token: verification.token,
      email: verification.email,
      expires_at: verification.expires_at,
      message: "Verification email resent successfully".to_string(),
    })),
    Err(e) => Err(Custom(
      Status::BadRequest,
      Json(ErrorResponse::from_auth_error(&e)),
    )),
  }
}
