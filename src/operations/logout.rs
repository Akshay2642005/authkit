use crate::auth::Auth;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Logout {
  pub token: String,
}

impl Logout {
  /// Constructs a `Logout` request containing the given token.
  ///
  /// # Examples
  ///
  /// ```
  /// let req = Logout::new("my-token");
  /// // `req` now holds the logout token to be sent to the logout endpoint.
  /// ```
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
    }
  }
}

impl From<&str> for Logout {
  /// Creates a `Logout` from a string slice.
  ///
  /// # Examples
  ///
  /// ```
  /// let _logout: Logout = Logout::from("my_token");
  /// ```
  fn from(token: &str) -> Self {
    Self::new(token)
  }
}

/// Invalidates the session associated with the provided logout token.
///
/// Attempts to delete the session identified by `request.token` from the persistent store and
/// returns success only if the session was removed.
///
/// # Returns
///
/// `Ok(())` if the session was successfully deleted, an error otherwise.
///
/// # Examples
///
/// ```no_run
/// # use crate::{Auth, Logout};
/// # async fn example(auth: &Auth) -> anyhow::Result<()> {
/// let req = Logout::new("some-token");
/// auth::execute(auth, req).await?;
/// # Ok(())
/// # }
/// ```
pub(crate) async fn execute(auth: &Auth, request: Logout) -> Result<()> {
  auth
    .inner
    .session_strategy
    .delete_session(auth.inner.db.as_ref().as_ref(), &request.token)
    .await?;

  Ok(())
}