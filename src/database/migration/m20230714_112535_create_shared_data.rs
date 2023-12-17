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
                    .table(SharedData::Table)
                    .if_not_exists()
                    // This table is keyed on the user that owns the data
                    .col(
                        ColumnDef::new(SharedData::UserId)
                            .unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    // ID of the currently active character for the user
                    .col(
                        ColumnDef::new(SharedData::ActiveCharacterId)
                            .unsigned()
                            .null(),
                    )
                    // Shared statistis about the user
                    .col(ColumnDef::new(SharedData::SharedStats).json().not_null())
                    // Shared equipment configuration
                    .col(
                        ColumnDef::new(SharedData::SharedEquipment)
                            .json()
                            .not_null(),
                    )
                    // Shared progression states
                    .col(
                        ColumnDef::new(SharedData::SharedProgression)
                            .json()
                            .not_null(),
                    )
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(SharedData::Table, SharedData::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the table
        manager
            .drop_table(Table::drop().table(SharedData::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SharedData {
    Table,
    UserId,
    ActiveCharacterId,
    SharedStats,
    SharedEquipment,
    SharedProgression,
}
