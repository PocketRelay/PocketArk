use sea_orm_migration::prelude::*;

use super::{
    m20230714_105755_create_users::Users,
    m20231223_184934_create_strike_team_missions::StrikeTeamMissions,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StrikeTeamMissionProgress::Table)
                    .if_not_exists()
                    // This table uses a composite key over the UserId and MissionId
                    .primary_key(
                        Index::create()
                            .col(StrikeTeamMissionProgress::UserId)
                            .col(StrikeTeamMissionProgress::MissionId),
                    )
                    .col(
                        ColumnDef::new(StrikeTeamMissionProgress::UserId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StrikeTeamMissionProgress::MissionId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StrikeTeamMissionProgress::UserMissionState)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StrikeTeamMissionProgress::Seen)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(StrikeTeamMissionProgress::Completed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StrikeTeamMissionProgress::Table,
                                StrikeTeamMissionProgress::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StrikeTeamMissionProgress::Table,
                                StrikeTeamMissionProgress::MissionId,
                            )
                            .to(StrikeTeamMissions::Table, StrikeTeamMissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StrikeTeamMissionProgress::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum StrikeTeamMissionProgress {
    Table,
    MissionId,
    UserId,
    UserMissionState,
    Seen,
    Completed,
}
