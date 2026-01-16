use crate::auth::{Auth, AuthInner};
use crate::error::{AuthError, Result};
use crate::strategies::password::PasswordStrategyType;
use crate::strategies::session::SessionStrategyType;
use crate::strategies::token::TokenStrategyType;
use crate::types::Database;
use std::sync::Arc;

pub struct AuthBuilder {
  database: Option<Database>,
  password_strategy: Option<PasswordStrategyType>,
  session_strategy: Option<SessionStrategyType>,
  token_strategy: Option<TokenStrategyType>,
}

impl AuthBuilder {
  /// Creates a new AuthBuilder with all configuration fields unset.
  ///
  /// # Examples
  ///
  /// ```
  /// let builder = AuthBuilder::new();
  /// ```
  pub(crate) fn new() -> Self {
    Self {
      database: None,
      password_strategy: None,
      session_strategy: None,
      token_strategy: None,
    }
  }
  /// Sets the database to be used by the builder.
  ///
  /// The provided `Database` is stored and will be consumed when `build()` is called.
  ///
  /// # Examples
  ///
  /// ```
  /// use crate::AuthBuilder;
  /// let db: Database = unimplemented!();
  /// let _ = AuthBuilder::new().database(db);
  /// ```
  pub fn database(mut self, db: Database) -> Self {
    self.database = Some(db);
    self
  }
  /// Sets the password strategy to use when building the Auth.
  ///
  /// # Parameters
  /// - `strategy`: The password strategy to apply; this replaces any previously set strategy.
  ///
  /// # Returns
  /// The updated `AuthBuilder`.
  ///
  /// # Examples
  ///
  /// ```
  /// let builder = AuthBuilder::new().password_strategy(PasswordStrategyType::Default);
  /// ```
  pub fn password_strategy(mut self, strategy: PasswordStrategyType) -> Self {
    self.password_strategy = Some(strategy);
    self
  }
  /// Sets the session strategy to use for the builder.
  ///
  /// This configures which session strategy will be used when constructing the `Auth` instance.
  ///
  /// # Returns
  ///
  /// The updated `AuthBuilder` with the provided session strategy set.
  ///
  /// # Examples
  ///
  /// ```
  /// use crate::SessionStrategyType;
  ///
  /// let builder = AuthBuilder::new()
  ///     .session_strategy(SessionStrategyType::default());
  /// ```
  pub fn session_strategy(mut self, strategy: SessionStrategyType) -> Self {
    self.session_strategy = Some(strategy);
    self
  }
  /// Sets the token strategy type to use for the `Auth` instance produced by `build`.
  ///
  /// `strategy` specifies which token generation/validation strategy the builder will configure.
  /// If this method is not called, `build()` will use the default `TokenStrategyType`.
  ///
  /// # Examples
  ///
  /// ```
  /// let builder = AuthBuilder::new().token_strategy(TokenStrategyType::default());
  /// ```
  pub fn token_strategy(mut self, strategy: TokenStrategyType) -> Self {
    self.token_strategy = Some(strategy);
    self
  }
  /// Builds an Auth instance from the configured database and strategies.
  ///
  /// Uses the provided strategies where set and falls back to defaults for any unspecified
  /// strategy. Validates that a database is provided and, when the `argon2` feature is not
  /// enabled, that a password strategy is explicitly supplied.
  ///
  /// # Errors
  ///
  /// Returns `AuthError::MissingDatabase` if no database was configured, or
  /// `AuthError::MissingPasswordStrategy` when a password strategy is required but absent.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// // Configure and build Auth (placeholder types shown for illustration)
  /// let db = /* obtain a Database value */ unimplemented!();
  /// let auth = AuthBuilder::new()
  ///     .database(db)
  ///     // optionally set strategies:
  ///     // .password_strategy(my_password_strategy)
  ///     // .session_strategy(my_session_strategy)
  ///     // .token_strategy(my_token_strategy)
  ///     .build();
  /// match auth {
  ///     Ok(a) => println!("Auth built successfully"),
  ///     Err(e) => eprintln!("Failed to build Auth: {:?}", e),
  /// }
  /// ```
  pub fn build(self) -> Result<Auth> {
    let database = self.database.ok_or(AuthError::MissingDatabase)?;

    #[cfg(feature = "argon2")]
    let password_strategy = self
      .password_strategy
      .unwrap_or_default()
      .create_strategy()?;

    #[cfg(not(feature = "argon2"))]
    let password_strategy = self
      .password_strategy
      .ok_or(AuthError::MissingPasswordStrategy)?
      .create_strategy()?;

    let session_strategy = self.session_strategy.unwrap_or_default().create_strategy();

    let db_trait = crate::database::create_database_trait(database.inner);
    let db_arc = Arc::new(db_trait);

    let token_strategy = self.token_strategy.unwrap_or_default().create_strategy();

    Ok(Auth {
      inner: Arc::new(AuthInner {
        db: db_arc,
        password_strategy,
        session_strategy,
        token_strategy,
      }),
    })
  }
}