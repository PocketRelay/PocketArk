use itertools::Itertools;
use sqlx::AssertSqlSafe;
use strum::IntoEnumIterator;

use crate::dto::currency::{CurrencyDto, CurrencyType, CurrencyUpdateDto};
use crate::dto::users::UserId;
use crate::{DbExecutor, DbResult};

pub struct CurrencyRepository;

impl CurrencyRepository {
    /// The maximum safe amount of currency to have before the game
    /// wraps it to a negative unusable amount
    pub const MAX_SAFE_CURRENCY: i32 = 100_000_000;

    /// Initialize a zero default currency for the user
    pub async fn add_initial_currency(db: impl DbExecutor<'_>, user_id: UserId) -> DbResult<()> {
        Self::apply_currency_updates(
            db,
            user_id,
            CurrencyType::iter()
                .map(|ty| CurrencyUpdateDto { ty, balance: 0 })
                .collect(),
        )
        .await
    }

    /// Get all currency instances for the provided user ID
    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<CurrencyDto>> {
        sqlx::query_as(r#"SELECT * FROM "currency" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_all(db)
            .await
    }

    /// Get a specific currency instance for the provided user ID
    pub async fn get_by_user_by_type(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        ty: CurrencyType,
    ) -> DbResult<Option<CurrencyDto>> {
        sqlx::query_as(r#"SELECT * FROM "currency" WHERE "user_id" = ? AND "ty" = ?"#)
            .bind(user_id)
            .bind(ty)
            .fetch_optional(db)
            .await
    }

    /// Sets the value of a currency for the provided user
    pub async fn set_currency_value(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        ty: CurrencyType,
        value: u32,
    ) -> DbResult<CurrencyDto> {
        let value = value.min(Self::MAX_SAFE_CURRENCY as u32);

        sqlx::query_as(
            r#"
            INSERT INTO "currency" ("user_id", "ty", "balance")
            VALUES (?, ?, ?)
            ON CONFLICT ("user_id", "ty")
                DO UPDATE SET "balance" = "excluded"."balance"
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(ty)
        .bind(value)
        .fetch_one(db)
        .await
    }

    /// Apply updates to the users currencies
    ///
    /// Enforces that the balance does not exceed the max safe limit for the game.
    ///
    /// Will create the currency instances if the user does not have them yet.
    pub async fn apply_currency_updates(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        mut updates: Vec<CurrencyUpdateDto>,
    ) -> DbResult<()> {
        // Empty updates set does nothing and would only produce an invalid query
        if updates.is_empty() {
            return Ok(());
        }

        // Enforce max safe currency rules
        updates
            .iter_mut()
            .for_each(|update| update.balance = update.balance.min(Self::MAX_SAFE_CURRENCY));

        // Every update needs 3 parameters for the "user_id", "ty", and "balance"
        let values_query = updates.iter().map(|_| "(?, ?, ?)").join(",");

        let mut query = sqlx::query(AssertSqlSafe(format!(
            r#"
        INSERT INTO "currency" ("user_id", "ty", "balance")
        VALUES {values_query}
        ON CONFLICT ("user_id", "ty")
            DO UPDATE SET "balance" = MAX(MIN("balance" + "excluded"."balance", ?), 0)
        "#,
        )));

        for update in updates {
            query = query.bind(user_id).bind(update.ty).bind(update.balance)
        }

        query.bind(Self::MAX_SAFE_CURRENCY).execute(db).await?;

        Ok(())
    }
}
