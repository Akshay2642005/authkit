use subtle::ConstantTimeEq;

/// Compares two strings for equality in constant time.
///
/// Returns `true` if the strings are exactly equal, `false` otherwise. If the lengths differ, the function returns `false` without performing a content-sensitive comparison.
///
/// # Examples
///
/// ```
/// assert!(constant_time_compare("secret", "secret"));
/// assert!(!constant_time_compare("secret", "Secret"));
/// assert!(!constant_time_compare("short", "longer"));
/// ```
#[allow(dead_code)]
pub fn constant_time_compare(a: &str, b: &str) -> bool {
  if a.len() != b.len() {
    return false;
  }

  a.as_bytes().ct_eq(b.as_bytes()).into()
}