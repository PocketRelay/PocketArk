use std::future::Future;

use crate::database::DbResult;
use sea_orm::entity::prelude::*;
use sea_orm::{IntoActiveModel, QuerySelect};

/// Type alias for a [u32] representing a user ID
pub type UserId = u32;

/// User database structure
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Unqiue ID for the account
    #[sea_orm(primary_key)]
    pub id: u32,
    /// Email address of the account
    pub email: String,
    /// Username for the account
    pub username: String,
    /// Password for the account
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::currency::Entity")]
    Currencies,
    #[sea_orm(has_many = "super::characters::Entity")]
    Characters,
    #[sea_orm(has_many = "super::inventory_items::Entity")]
    InventoryItems,
    #[sea_orm(has_many = "super::seen_articles::Entity")]
    SeenArticles,
    #[sea_orm(has_many = "super::challenge_progress::Entity")]
    ChallengeProgress,
    #[sea_orm(has_one = "super::shared_data::Entity")]
    SharedData,
    #[sea_orm(has_many = "super::strike_teams::Entity")]
    StrikeTeams,
}

/// Partial structure for creating a new user
#[derive(DeriveIntoActiveModel)]
pub struct CreateUser {
    /// The email to give the user
    pub email: String,
    /// The username to give the user
    pub username: String,
    /// The password to give the user
    pub password: String,
}

impl Model {
    /// Creates a new user from the provided [CreateUser] structure
    pub fn create<C>(
        db: &C,
        mut create: CreateUser,
    ) -> impl Future<Output = DbResult<Self>> + Send + '_
    where
        C: ConnectionTrait + Send,
    {
        // Emails are stored in lowercase to be case-insensitive
        create.email = create.email.to_lowercase();

        create.into_active_model().insert(db)
    }

    /// Checks if an account with a matching `username` already
    /// exists in the database
    pub async fn username_exists<'db, C>(db: &C, username: &str) -> DbResult<bool>
    where
        C: ConnectionTrait + Send,
    {
        let result: Option<String> = Entity::find()
            .select_only()
            .column(Column::Username)
            // Match against the email
            .filter(Column::Username.eq(username))
            .into_tuple()
            .one(db)
            .await?;

        Ok(result.is_some())
    }

    /// Checks if an account with a matching `email` already
    /// exists in the database
    pub async fn email_exists<'db, C>(db: &C, email: &str) -> DbResult<bool>
    where
        C: ConnectionTrait + Send,
    {
        // Emails are stored in lowercase to be case-insensitive
        let email_lower = email.to_lowercase();

        let result: Option<String> = Entity::find()
            .select_only()
            .column(Column::Email)
            // Match against the email
            .filter(Column::Email.eq(email_lower))
            .into_tuple()
            .one(db)
            .await?;

        Ok(result.is_some())
    }

    /// Finds a user by its [UserId]
    pub fn by_id<C>(db: &C, id: UserId) -> impl Future<Output = DbResult<Option<Self>>> + Send + '_
    where
        C: ConnectionTrait + Send,
    {
        Entity::find_by_id(id).one(db)
    }

    /// Finds a user by its `email`
    pub fn by_email<'db, C>(
        db: &'db C,
        email: &str,
    ) -> impl Future<Output = DbResult<Option<Self>>> + Send + 'db
    where
        C: ConnectionTrait + Send,
    {
        // Emails are stored in lowercase to be case-insensitive
        let email_lower = email.to_lowercase();

        Entity::find()
            // Match against the email
            .filter(Column::Email.eq(email_lower))
            .one(db)
    }
}

impl Related<super::currency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Currencies.def()
    }
}

impl Related<super::characters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Characters.def()
    }
}

impl Related<super::inventory_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InventoryItems.def()
    }
}

impl Related<super::seen_articles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SeenArticles.def()
    }
}

impl Related<super::shared_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SharedData.def()
    }
}

impl Related<super::challenge_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChallengeProgress.def()
    }
}

impl Related<super::strike_teams::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StrikeTeams.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
