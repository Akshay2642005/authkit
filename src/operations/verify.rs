use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::User;

#[derive(Debug, Clone)]
pub struct Verify {
  pub token: String,
}

impl Verify {
  /// Creates a `Verify` containing the provided token string.
  ///
  /// The `token` argument is converted into a `String` and stored in the returned `Verify`.
  ///
  /// # Examples
  ///
  /// ```
  /// let v = Verify::new("token123");
  /// ```
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
    }
  }
}

impl From<&str> for Verify {
  /// Create a `Verify` value from a token string slice.
  ///
  /// # Examples
  ///
  /// ```
  /// let v: Verify = Verify::from("token");
  /// let v2 = Verify::new("token");
  /// ```
  fn from(token: &str) -> Self {
    Self::new(token)
  }
}

/// Verifies a session token and returns the associated user.
///
/// Returns `AuthError::InvalidSession` if the token does not correspond to an active session
/// or if the session has expired. Returns `AuthError::UserNotFound` if the session exists but
/// the referenced user cannot be found.
///
/// # Examples
///
/// ```no_run
/// # async {
/// // `auth` should be an initialized `Auth` instance.
/// let user = execute(&auth, Verify::new("some-token")).await.unwrap();
/// // use `user`
/// # };
/// ```
pub(crate) async fn execute(auth: &Auth, request: Verify) -> Result<User> {
  let session = auth
    .inner
    .session_strategy
    .find_session(auth.inner.db.as_ref().as_ref(), &request.token)
    .await?
    .ok_or(AuthError::InvalidSession)?;

  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  if session.expires_at < now {
    return Err(AuthError::InvalidSession);
  }

  auth
    .inner
    .db
    .find_user_by_id(&session.user_id)
    .await?
    .ok_or(AuthError::UserNotFound)
}