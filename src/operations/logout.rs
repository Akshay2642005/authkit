use crate::auth::Auth;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Logout {
	pub token: String,
}

impl Logout {
	pub fn new(token: impl Into<String>) -> Self {
		Self {
			token: token.into(),
		}
	}
}

impl From<&str> for Logout {
	fn from(token: &str) -> Self {
		Self::new(token)
	}
}

pub(crate) async fn execute(auth: &Auth, request: Logout) -> Result<()> {
	auth
		.inner
		.session_strategy
		.delete_session(auth.inner.db.as_ref().as_ref(), &request.token)
		.await?;

	Ok(())
}
