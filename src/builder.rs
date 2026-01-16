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
  pub(crate) fn new() -> Self {
    Self {
      database: None,
      password_strategy: None,
      session_strategy: None,
      token_strategy: None,
    }
  }
  pub fn database(mut self, db: Database) -> Self {
    self.database = Some(db);
    self
  }
  pub fn password_strategy(mut self, strategy: PasswordStrategyType) -> Self {
    self.password_strategy = Some(strategy);
    self
  }
  pub fn session_strategy(mut self, strategy: SessionStrategyType) -> Self {
    self.session_strategy = Some(strategy);
    self
  }
  pub fn token_strategy(mut self, strategy: TokenStrategyType) -> Self {
    self.token_strategy = Some(strategy);
    self
  }
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
