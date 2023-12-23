use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StrikeTeamMissions::Table)
                    .if_not_exists()
                    // Unique ID of the strike team mission
                    .col(
                        ColumnDef::new(StrikeTeamMissions::Id)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // The mission descriptor details
                    .col(
                        ColumnDef::new(StrikeTeamMissions::Descriptor)
                            .json()
                            .not_null(),
                    )
                    // The mission type details
                    .col(
                        ColumnDef::new(StrikeTeamMissions::MissionType)
                            .json()
                            .not_null(),
                    )
                    // Mission accessiblity
                    .col(
                        ColumnDef::new(StrikeTeamMissions::Accessibility)
                            .unsigned()
                            .not_null(),
                    )
                    // Custom defined mission waves
                    .col(ColumnDef::new(StrikeTeamMissions::Waves).json().not_null())
                    // Mission tags
                    .col(ColumnDef::new(StrikeTeamMissions::Tags).json().not_null())
                    // Static mission modifiers
                    .col(
                        ColumnDef::new(StrikeTeamMissions::StaticModifiers)
                            .json()
                            .not_null(),
                    )
                    // Dynamic mission modifiers
                    .col(
                        ColumnDef::new(StrikeTeamMissions::DynamicModifiers)
                            .json()
                            .not_null(),
                    )
                    // The mission rewarads
                    .col(
                        ColumnDef::new(StrikeTeamMissions::Rewards)
                            .json()
                            .not_null(),
                    )
                    // Custom attributes associated with the mission
                    .col(
                        ColumnDef::new(StrikeTeamMissions::CustomAttributes)
                            .json()
                            .not_null(),
                    )
                    // The time in seconds when the mission became available
                    .col(
                        ColumnDef::new(StrikeTeamMissions::StartSeconds)
                            .unsigned()
                            .not_null(),
                    )
                    // The time in seconds when the mission is no longer available
                    .col(
                        ColumnDef::new(StrikeTeamMissions::EndSeconds)
                            .unsigned()
                            .not_null(),
                    )
                    // The time in seconds the mission will take to complete (Strike teams)
                    .col(
                        ColumnDef::new(StrikeTeamMissions::SpLengthSeconds)
                            .unsigned()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StrikeTeamMissions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum StrikeTeamMissions {
    Table,
    Id,
    Descriptor,
    MissionType,
    Accessibility,
    Waves,
    Tags,
    StaticModifiers,
    DynamicModifiers,
    Rewards,
    CustomAttributes,
    StartSeconds,
    EndSeconds,
    SpLengthSeconds,
}
