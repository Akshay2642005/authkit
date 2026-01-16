use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::User;
use crate::validation;

#[derive(Debug, Clone)]
pub struct Register {
  pub email: String,
  pub password: String,
}

pub(crate) async fn execute(auth: &Auth, request: Register) -> Result<User> {
  validation::email::validate(&request.email)?;

  validation::password::validate(&request.password)?;

  if let Some(_existing) = auth.inner.db.find_user_by_email(&request.email).await? {
    return Err(AuthError::UserAlreadyExists(request.email));
  }
  let password_hash = auth
    .inner
    .password_strategy
    .hash_password(&request.password)
    .await?;

  let user_id = crate::security::tokens::generate_id();

  let created_at = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  let user = auth
    .inner
    .db
    .create_user(&user_id, &request.email, &password_hash, created_at)
    .await?;

  Ok(user)
}
