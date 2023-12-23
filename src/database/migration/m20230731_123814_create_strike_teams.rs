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
                    // Unqiue ID for this strike team
                    .col(
                        ColumnDef::new(StrikeTeams::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    // ID of the user that owns this strike team
                    .col(ColumnDef::new(StrikeTeams::UserId).unsigned().not_null())
                    // Name of the strike team (Shown in game)
                    .col(ColumnDef::new(StrikeTeams::Name).string().not_null())
                    // Icon to use with the strike team
                    .col(ColumnDef::new(StrikeTeams::Icon).json().not_null())
                    // Current level of the strike team
                    .col(ColumnDef::new(StrikeTeams::Level).unsigned().not_null())
                    // XP progression for the strike team
                    .col(ColumnDef::new(StrikeTeams::Xp).json().not_null())
                    // Equipment if the strike team has one active
                    .col(ColumnDef::new(StrikeTeams::Equipment).json().null())
                    // Positive traits this strike team has
                    .col(
                        ColumnDef::new(StrikeTeams::PositiveTraits)
                            .json()
                            .not_null(),
                    )
                    // Negative traits this strike team has
                    .col(
                        ColumnDef::new(StrikeTeams::NegativeTraits)
                            .json()
                            .not_null(),
                    )
                    // Unknown usage
                    .col(
                        ColumnDef::new(StrikeTeams::OutOfDate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(StrikeTeams::Table, StrikeTeams::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create a unique index accross the user ID
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx-strike-team-uid")
                    .table(StrikeTeams::Table)
                    .col(StrikeTeams::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StrikeTeams::Table).to_owned())
            .await?;

        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(StrikeTeams::Table)
                    .name("idx-strike-team-uid")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum StrikeTeams {
    Table,
    Id,
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
