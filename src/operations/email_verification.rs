use crate::{auth::Auth, error::Result, types::VerificationToken};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SendEmailVerification {
	pub user_id: String,
	pub to_email: String,
	pub redirect_url: Option<String>,
	pub token: Option<String>,
}

#[allow(dead_code)]
pub(crate) async fn send_email_verification(
	_auth: &Auth,
	_request: SendEmailVerification,
) -> Result<VerificationToken> {
	todo!("Not implemented yet")
}

#[allow(dead_code)]
pub struct VerifyEmail {
	pub token: String,
}

#[allow(dead_code)]
pub(crate) async fn verify_email(_auth: &Auth, _request: VerifyEmail) -> Result<()> {
	todo!("Not implemented yet")
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ResendEmailVerification {
	email: String,
}

#[allow(dead_code)]
pub(crate) async fn resend_email_verification(
	_auth: &Auth,
	_request: ResendEmailVerification,
) -> Result<VerificationToken> {
	todo!("Not implemented yet")
}
