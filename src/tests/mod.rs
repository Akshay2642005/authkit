//! Comprehensive test suite for AuthKit
//!
//! This module contains tests for:
//! - Validation (email, password)
//! - Authentication operations (register, login, verify, logout)
//! - Error handling and edge cases
//! - Session management
//! - Security features

// Only compile tests when at least one database feature is enabled
#[cfg(any(feature = "sqlite", feature = "postgres"))]
mod error_tests;

#[cfg(any(feature = "sqlite", feature = "postgres"))]
mod integration_tests;

mod validation_tests;
