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
                    .col(
                        ColumnDef::new(SharedData::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(SharedData::UserId).unsigned().not_null())
                    .col(
                        ColumnDef::new(SharedData::ActiveCharacterId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SharedData::SharedStats).json().not_null())
                    .col(
                        ColumnDef::new(SharedData::SharedEquipment)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SharedData::SharedProgression)
                            .json()
                            .not_null(),
                    )
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
        manager
            .drop_table(Table::drop().table(SharedData::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum SharedData {
    Table,
    Id,
    UserId,
    ActiveCharacterId,
    SharedStats,
    SharedEquipment,
    SharedProgression,
}
