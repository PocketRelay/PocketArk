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
                    .table(Item::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Item::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Item::UserId).unsigned().not_null())
                    .col(ColumnDef::new(Item::DefinitionName).uuid().not_null())
                    .col(ColumnDef::new(Item::StackSize).unsigned().not_null())
                    .col(ColumnDef::new(Item::Seen).boolean().not_null())
                    .col(ColumnDef::new(Item::InstanceAttributes).json().not_null())
                    .col(ColumnDef::new(Item::Created).date_time().not_null())
                    .col(ColumnDef::new(Item::LastGrant).date_time().not_null())
                    .col(ColumnDef::new(Item::EarnedBy).string().not_null())
                    .col(ColumnDef::new(Item::Restricted).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Item::Table, Item::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Item::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Item {
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
