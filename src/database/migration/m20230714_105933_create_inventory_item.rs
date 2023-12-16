use sea_orm_migration::prelude::*;

use super::m20230714_105755_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the inventory items table
        manager
            .create_table(
                Table::create()
                    .table(InventoryItems::Table)
                    .if_not_exists()
                    // Unique ID for this item
                    .col(
                        ColumnDef::new(InventoryItems::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    // ID of the user this item belongs to
                    .col(ColumnDef::new(InventoryItems::UserId).unsigned().not_null())
                    // Name of the definition this item instance belongs to (Unqiue on a per user basis)
                    .col(
                        ColumnDef::new(InventoryItems::DefinitionName)
                            .uuid()
                            .not_null(),
                    )
                    // Size of this item stack
                    .col(
                        ColumnDef::new(InventoryItems::StackSize)
                            .unsigned()
                            .not_null(),
                    )
                    // Whether the user has seen this item
                    .col(
                        ColumnDef::new(InventoryItems::Seen)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Attributes unique to this item instance
                    .col(
                        ColumnDef::new(InventoryItems::InstanceAttributes)
                            .json()
                            .not_null(),
                    )
                    // The date and time this item was created
                    .col(
                        ColumnDef::new(InventoryItems::Created)
                            .date_time()
                            .not_null(),
                    )
                    // The last time this item was granted to the player
                    .col(
                        ColumnDef::new(InventoryItems::LastGrant)
                            .date_time()
                            .not_null(),
                    )
                    // The reason the player earned this item
                    .col(ColumnDef::new(InventoryItems::EarnedBy).string().not_null())
                    // Whether this item is restricted
                    .col(
                        ColumnDef::new(InventoryItems::Restricted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryItems::Table, InventoryItems::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create a unique index accross the user ID and item definition
        // (Users should only have a single item per definition)
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx-item-uid-def")
                    .table(InventoryItems::Table)
                    .col(InventoryItems::UserId)
                    .col(InventoryItems::DefinitionName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the table
        manager
            .drop_table(Table::drop().table(InventoryItems::Table).to_owned())
            .await?;

        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(InventoryItems::Table)
                    .name("idx-item-uid-def")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum InventoryItems {
    Table,
    Id,
    UserId,
    DefinitionName,
    StackSize,
    Seen,
    InstanceAttributes,
    Created,
    LastGrant,
    EarnedBy,
    Restricted,
}
