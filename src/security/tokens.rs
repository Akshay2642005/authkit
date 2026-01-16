use rand::RngCore;

const TOKEN_LENGTH: usize = 32;
const ID_LENGTH: usize = 16;

/// Generate a secure random session token.
///
/// Returns a hex-encoded string representing 32 random bytes (64 hex characters).
///
/// # Examples
///
/// ```
/// let token = generate_token();
/// assert_eq!(token.len(), 64);
/// ```
pub fn generate_token() -> String {
  let mut rng = rand::rng();
  let mut bytes = vec![0u8; TOKEN_LENGTH];
  rng.fill_bytes(&mut bytes);
  hex::encode(bytes)
}

/// Generates a hex-encoded identifier composed of ID_LENGTH cryptographically secure random bytes.
///
/// The returned string is the lowercase hexadecimal encoding of ID_LENGTH random bytes.
///
/// # Examples
///
/// ```
/// let id = crate::generate_id();
/// // Hex string length should be twice the number of bytes
/// assert_eq!(hex::decode(&id).unwrap().len(), crate::ID_LENGTH);
/// ```
pub fn generate_id() -> String {
  let mut rng = rand::rng();
  let mut bytes = vec![0u8; ID_LENGTH];
  rng.fill_bytes(&mut bytes);
  hex::encode(bytes)
}