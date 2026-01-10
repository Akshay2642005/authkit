use rand::RngCore;

const TOKEN_LENGTH: usize = 32;
const ID_LENGTH: usize = 16;

/// Generate a secure random token for sessions
pub fn generate_token() -> String {
	let mut rng = rand::rng();
	let mut bytes = vec![0u8; TOKEN_LENGTH];
	rng.fill_bytes(&mut bytes);
	hex::encode(bytes)
}

/// Generate a secure random ID for users
pub fn generate_id() -> String {
	let mut rng = rand::rng();
	let mut bytes = vec![0u8; ID_LENGTH];
	rng.fill_bytes(&mut bytes);
	hex::encode(bytes)
}
