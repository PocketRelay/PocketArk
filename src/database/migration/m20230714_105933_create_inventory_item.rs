use sea_orm_migration::prelude::*;

use super::m20230714_105755_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InventoryItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryItems::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(InventoryItems::ItemId)
                            .uuid()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InventoryItems::UserId).unsigned().not_null())
                    .col(
                        ColumnDef::new(InventoryItems::DefinitionName)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryItems::StackSize)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InventoryItems::Seen).boolean().not_null())
                    .col(
                        ColumnDef::new(InventoryItems::InstanceAttributes)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryItems::Created)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryItems::LastGrant)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InventoryItems::EarnedBy).string().not_null())
                    .col(
                        ColumnDef::new(InventoryItems::Restricted)
                            .boolean()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryItems::Table, InventoryItems::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InventoryItems::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum InventoryItems {
    Table,
    Id,
    ItemId,
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
