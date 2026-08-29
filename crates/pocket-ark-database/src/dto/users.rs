use sqlx::prelude::FromRow;

pub type UserId = i64;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct UserDto {
    /// Unique ID for the account
    pub id: UserId,
    /// Email address of the account
    pub email: String,
    /// Username for the account
    pub username: String,
    /// Password for the account
    pub password: String,
}

pub struct CreateUserDto {
    /// The email to give the user
    pub email: NormalizedEmail,
    /// The username to give the user
    pub username: String,
    /// The password to give the user
    pub password: String,
}

pub struct NormalizedEmail(String);

impl NormalizedEmail {
    pub fn new(email: impl AsRef<str>) -> Self {
        let email: &str = email.as_ref();
        let value = email.trim().to_lowercase();
        NormalizedEmail(value)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<str> for NormalizedEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
