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
                    .table(StrikeTeams::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StrikeTeams::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(StrikeTeams::TeamId)
                            .uuid()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StrikeTeams::UserId).unsigned().not_null())
                    .col(ColumnDef::new(StrikeTeams::Name).string().not_null())
                    .col(ColumnDef::new(StrikeTeams::Icon).json().not_null())
                    .col(ColumnDef::new(StrikeTeams::Level).unsigned().not_null())
                    .col(ColumnDef::new(StrikeTeams::Xp).json().not_null())
                    .col(ColumnDef::new(StrikeTeams::Equipment).json().nullable())
                    .col(
                        ColumnDef::new(StrikeTeams::PositiveTraits)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StrikeTeams::NegativeTraits)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StrikeTeams::OutOfDate).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(StrikeTeams::Table, StrikeTeams::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StrikeTeams::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum StrikeTeams {
    Table,
    Id,
    TeamId,
    UserId,
    Name,
    Icon,
    Level,
    Xp,
    Equipment,
    PositiveTraits,
    NegativeTraits,
    OutOfDate,
}
