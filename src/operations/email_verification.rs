use crate::{auth::Auth, error::Result, types::VerificationToken};
#[derive(Debug, Clone)]
pub struct SendEmailVerification {
	pub user_id: String,
	pub to_email: String,
	pub redirect_url: Option<String>,
	pub token: Option<String>,
}

pub(crate) async fn send_email_verification(
	auth: &Auth,
	request: SendEmailVerification,
) -> Result<VerificationToken> {
	todo!("Not implemented yet")
}

pub struct VerifyEmail {
	pub token: String,
}

pub(crate) async fn verify_email(auth: &Auth, request: VerifyEmail) -> Result<()> {
	todo!("Not implemented yet")
}

#[derive(Clone, Debug)]
pub struct ResendEmailVerification {
	email: String,
}

pub(crate) async fn resend_email_verification(
	auth: &Auth,
	request: ResendEmailVerification,
) -> Result<VerificationToken> {
	todo!("Not implemented yet")
}
