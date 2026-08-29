use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
pub struct MigrationDto {
    pub name: String,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateMigrationDto {
    pub name: String,
    pub applied_at: DateTime<Utc>,
}
