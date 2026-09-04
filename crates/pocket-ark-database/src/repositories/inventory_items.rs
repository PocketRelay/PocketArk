use itertools::Itertools;
use sqlx::AssertSqlSafe;
use uuid::Uuid;

use crate::{
    DbExecutor, DbResult,
    dto::{
        inventory_items::{CreateInventoryItemDto, InventoryItemDto, InventoryItemEarnedBy},
        users::UserId,
    },
    extensions::SqlxBindExt,
};

pub struct InventoryItemsRepository;

impl InventoryItemsRepository {
    /// Add an item to a players inventory, if the item already exists in the
    /// players inventory their capacity will be expanded
    pub async fn add_item(
        db: impl DbExecutor<'_>,
        create: CreateInventoryItemDto,
    ) -> DbResult<InventoryItemDto> {
        let item_id = Uuid::new_v4();
        let capacity = create.capacity.unwrap_or(u32::MAX);

        sqlx::query_as(
            r#"
            INSERT INTO "inventory_items" (
                "item_id", "user_id", "definition_name", "stack_size",
                "instance_attributes", "created", "last_grant", "earned_by"
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT ("user_id", "definition_name")
                DO UPDATE SET
                    "stack_size" = MIN("stack_size" + "excluded"."stack_size", ?),
                    "last_grant" = "excluded"."last_grant"
            RETURNING *
            "#,
        )
        .bind(item_id)
        .bind(create.user_id)
        .bind(create.definition_name)
        .bind(create.stack_size)
        .bind("{}")
        .bind(create.created_at)
        .bind(create.created_at)
        .bind(InventoryItemEarnedBy::Granted)
        .bind(capacity)
        .fetch_one(db)
        .await
    }

    /// Sets the size of an item stack for the provided user
    pub async fn set_item_stack_size(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        definition_name: Uuid,
        stack_size: u32,
    ) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE "inventory_items"
            SET "stack_size" = ?
            WHERE "user_id" = ? AND "definition_name" = ?
        "#,
        )
        .bind(stack_size)
        .bind(user_id)
        .bind(definition_name)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark a collection of items as seen by their item definition names
    pub async fn mark_items_seen(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        items: &[Uuid],
    ) -> DbResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let placeholders = items.iter().map(|_| "?").join(",");
        let query = format!(
            r#"
        UPDATE "inventory_items"
        SET "seen" = TRUE
        WHERE "user_id" = ? AND "definition_name" IN ({placeholders})
        "#
        );

        // This query is asserted to be safe as items is a set of uuids parsed and validated
        // and thus its not possible to insert any custom characters into the query
        let query = AssertSqlSafe(query);
        sqlx::query(query)
            .bind(user_id)
            .bind_all(items)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Get all items for a user
    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<InventoryItemDto>> {
        sqlx::query_as(r#"SELECT * FROM "inventory_items" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_all(db)
            .await
    }

    /// Get a specific item for a user by item ID
    pub async fn get_by_user_by_item_id(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        item_id: Uuid,
    ) -> DbResult<Option<InventoryItemDto>> {
        sqlx::query_as(r#"SELECT * FROM "inventory_items" WHERE "user_id" = ? AND "item_id" = ?"#)
            .bind(user_id)
            .bind(item_id)
            .fetch_optional(db)
            .await
    }

    /// Get all items for a user that are within the set of item definition names
    pub async fn get_by_user_by_definitions(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        definition_names: &[Uuid],
    ) -> DbResult<Vec<InventoryItemDto>> {
        if definition_names.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = definition_names.iter().map(|_| "?").join(",");
        let query = format!(
            r#"
        SELECT * FROM "inventory_items"
        WHERE "user_id" = ? AND "definition_name" IN ({placeholders})
        "#
        );

        // No user generated data is added to the query only the placeholders for
        // the bound values of dynamic length
        let query = AssertSqlSafe(query);
        sqlx::query_as(query)
            .bind(user_id)
            .bind_all(definition_names)
            .fetch_all(db)
            .await
    }
}
