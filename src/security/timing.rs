use subtle::ConstantTimeEq;

/// Timing-safe string comparison
#[allow(dead_code)]
pub fn constant_time_compare(a: &str, b: &str) -> bool {
  if a.len() != b.len() {
    return false;
  }

  a.as_bytes().ct_eq(b.as_bytes()).into()
}
