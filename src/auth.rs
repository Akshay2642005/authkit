use crate::database::DatabaseTrait;
use crate::error::Result;
use crate::operations::{Login, Logout, Register, Verify};
use crate::strategies::password::PasswordStrategy;
use crate::strategies::session::SessionStrategy;
use crate::types::{Session, User};
use std::sync::Arc;

#[derive(Clone)]
pub struct Auth {
	pub(crate) inner: Arc<AuthInner>,
}

pub(crate) struct AuthInner {
	pub(crate) db: Box<dyn DatabaseTrait>,
	pub(crate) password_strategy: Box<dyn PasswordStrategy>,
	pub(crate) session_strategy: Box<dyn SessionStrategy>,
}

impl Auth {
	pub fn builder() -> crate::builder::AuthBuilder {
		crate::builder::AuthBuilder::new()
	}
	pub async fn register(&self, request: Register) -> Result<User> {
		crate::operations::register::execute(self, request).await
	}
	pub async fn login(&self, request: Login) -> Result<Session> {
		crate::operations::login::execute(self, request).await
	}
	pub async fn verify(&self, request: Verify) -> Result<User> {
		crate::operations::verify::execute(self, request).await
	}
	pub async fn logout(&self, request: Logout) -> Result<()> {
		crate::operations::logout::execute(self, request).await
	}
	pub async fn migrate(&self) -> Result<()> {
		self.inner.db.migrate().await
	}
}

unsafe impl Send for Auth {}
unsafe impl Sync for Auth {}
