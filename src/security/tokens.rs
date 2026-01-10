use rand::Rng;

const TOKEN_LENGTH: usize = 32;
const ID_LENGTH: usize = 16;

/// Generate a secure random token for sessions
pub fn generate_token() -> String {
	let mut rng = rand::thread_rng();
	let bytes: Vec<u8> = (0..TOKEN_LENGTH).map(|_| rng.r#gen::<u8>()).collect();
	hex::encode(bytes)
}

/// Generate a secure random ID for users
pub fn generate_id() -> String {
	let mut rng = rand::thread_rng();
	let bytes: Vec<u8> = (0..ID_LENGTH).map(|_| rng.r#gen::<u8>()).collect();
	hex::encode(bytes)
}
