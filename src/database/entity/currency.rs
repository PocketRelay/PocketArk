use super::{users::UserId, Currency, User};
use crate::database::DbResult;
use sea_orm::{
    entity::prelude::*, sea_query::OnConflict, ActiveValue::Set, InsertResult, IntoActiveModel,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_with::{DeserializeAs, DisplayFromStr};
use std::{fmt::Display, future::Future, str::FromStr};

/// Currency database structure
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "currency")]
pub struct Model {
    // ID of the user this currency data belongs to
    #[sea_orm(primary_key)]
    pub user_id: UserId,
    // The type of the currency
    #[sea_orm(primary_key)]
    pub ty: CurrencyType,
    // The amount of currency the user has
    pub balance: u32,
}

/// Enum for the different known currency types
#[derive(Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[repr(u8)]
pub enum CurrencyType {
    Mtx = 0,
    Grind = 1,
    Mission = 2,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// The maximum safe amount of currency to have before the game
    /// wraps it to a negative unusable amount
    pub const MAX_SAFE_CURRENCY: u32 = 100_000_000;

    /// Sets the default currency values for the provided `user`
    pub fn set_default<'db, C>(
        db: &'db C,
        user: &User,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Self::set_many(
            db,
            user,
            [
                (CurrencyType::Mtx, 0),
                (CurrencyType::Grind, 0),
                (CurrencyType::Mission, 0),
            ],
        )
    }

    /// Conflict strategy for replacing the existing blance
    /// when a balance exists
    fn set_balance_conflict() -> OnConflict {
        // Update the value column if a key already exists
        OnConflict::columns([Column::UserId, Column::Ty])
            // Update the balance value
            .update_column(Column::Balance)
            .to_owned()
    }

    /// Sets the balance of a specific `ty` currency to `value`
    /// for the specific `user`. Will create the currency if it
    /// doesn't exist.
    pub fn set<'db, C>(
        db: &'db C,
        user: &User,
        ty: CurrencyType,
        value: u32,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::insert(ActiveModel {
            user_id: Set(user.id),
            ty: Set(ty),
            balance: Set(value),
        })
        .on_conflict(Self::set_balance_conflict())
        .exec(db)
    }

    /// Sets multiple currency balances from an iterator of [CurrencyName] and
    /// balance pairs.
    pub fn set_many<'db, C, I>(
        db: &'db C,
        user: &User,
        values: I,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + 'db
    where
        C: ConnectionTrait + Send,
        I: IntoIterator<Item = (CurrencyType, u32)>,
    {
        Entity::insert_many(values.into_iter().map(|(ty, value)| ActiveModel {
            user_id: Set(user.id),
            ty: Set(ty),
            balance: Set(value),
        }))
        .on_conflict(Self::set_balance_conflict())
        .exec(db)
    }

    /// Updates the currency balance setting it to the provided `amount`
    pub fn update<C>(self, db: &C, amount: u32) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.balance = Set(amount);
        model.update(db)
    }

    /// Finds all the currency entities for the provided `user`
    pub fn all<'db, C>(
        db: &'db C,
        user: &User,
    ) -> impl Future<Output = DbResult<Vec<Currency>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db)
    }

    /// Gets a specific currency type for this user
    pub fn get<'db, C>(
        db: &'db C,
        user: &User,
        ty: CurrencyType,
    ) -> impl Future<Output = DbResult<Option<Currency>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).filter(Column::Ty.eq(ty)).one(db)
    }

    /// Conflict strategy for adding the balancing onto
    /// an existing balance
    fn add_balance_conflict() -> OnConflict {
        // Update the value column if a key already exists
        OnConflict::columns([Column::UserId, Column::Ty])
            .value(
                Column::Balance,
                // Adds the balance to the existing balance without surpassing
                // the safe currency limit
                Expr::cust_with_values(
                    "(SELECT MIN(`balance` + `excluded`.`balance`, ?))",
                    [Self::MAX_SAFE_CURRENCY],
                ),
            )
            .to_owned()
    }

    /// Adds an amount to a specific currency balance
    pub fn add<'db, C>(
        db: &'db C,
        user: &User,
        ty: CurrencyType,
        amount: u32,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        Entity::insert(ActiveModel {
            user_id: Set(user.id),
            ty: Set(ty),
            balance: Set(amount),
        })
        .on_conflict(Self::add_balance_conflict())
        .exec(db)
    }

    /// Adds an amount to multiple balances
    pub fn add_many<'db, C, I>(
        db: &'db C,
        user: &User,
        values: I,
    ) -> impl Future<Output = DbResult<InsertResult<ActiveModel>>> + 'db
    where
        C: ConnectionTrait + Send,
        I: IntoIterator<Item = (CurrencyType, u32)>,
    {
        Entity::insert_many(values.into_iter().map(|(ty, value)| ActiveModel {
            user_id: Set(user.id),
            ty: Set(ty),
            balance: Set(value),
        }))
        .on_conflict(Self::add_balance_conflict())
        .exec(db)
    }
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_struct("Currency", 2)?;
        value.serialize_field("name", &self.ty)?;
        value.serialize_field("balance", &self.balance)?;
        value.end()
    }
}

impl Display for CurrencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CurrencyType::Mtx => "MTXCurrency",
            CurrencyType::Grind => "GrindCurrency",
            CurrencyType::Mission => "MissionCurrency",
        })
    }
}

/// Unknown currency error
#[derive(Debug)]
pub struct UnknownCurrency;

impl FromStr for CurrencyType {
    type Err = UnknownCurrency;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "MTXCurrency" => Self::Mtx,
            "GrindCurrency" => Self::Grind,
            "MissionCurrency" => Self::Mission,
            _ => return Err(UnknownCurrency),
        })
    }
}

impl std::error::Error for UnknownCurrency {}

impl Display for UnknownCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unknown currency type")
    }
}

impl Serialize for CurrencyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CurrencyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}
