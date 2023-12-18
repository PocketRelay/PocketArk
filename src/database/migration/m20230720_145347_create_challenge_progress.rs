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
                    .table(ChallengeProgress::Table)
                    .if_not_exists()
                    // This table uses a composite key over the UserId and ChallengeId
                    .primary_key(
                        Index::create()
                            .col(ChallengeProgress::UserId)
                            .col(ChallengeProgress::ChallengeId),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::UserId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::ChallengeId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::Counters)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChallengeProgress::State).string().not_null())
                    .col(
                        ColumnDef::new(ChallengeProgress::TimesCompleted)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::LastCompleted)
                            .date_time()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::FirstCompleted)
                            .date_time()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::LastChanged)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChallengeProgress::Rewarded)
                            .boolean()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChallengeProgress::Table, ChallengeProgress::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChallengeProgress::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ChallengeProgress {
    Table,
    Id,
    ChallengeId,
    UserId,
    Counters,
    State,
    TimesCompleted,
    LastCompleted,
    FirstCompleted,
    LastChanged,
    Rewarded,
}
