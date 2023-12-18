use sea_orm_migration::prelude::*;

use super::{
    m20230714_105755_create_users::Users,
    m20230720_145347_create_challenge_progress::ChallengeProgress,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the class data table
        manager
            .create_table(
                Table::create()
                    .table(ChallengeCounter::Table)
                    .if_not_exists()
                    // This table uses a composite key over the UserId, ChallengeId, and Name
                    .primary_key(
                        Index::create()
                            .col(ChallengeCounter::UserId)
                            .col(ChallengeCounter::ChallengeId)
                            .col(ChallengeCounter::Name),
                    )
                    // ID of the user this data belongs to
                    .col(
                        ColumnDef::new(ChallengeCounter::UserId)
                            .unsigned()
                            .not_null(),
                    )
                    // Name of the challenge this counter is for
                    .col(
                        ColumnDef::new(ChallengeCounter::ChallengeId)
                            .uuid()
                            .not_null(),
                    )
                    // Name of the counter
                    .col(ColumnDef::new(ChallengeCounter::Name).string().not_null())
                    // The number of times completed
                    .col(
                        ColumnDef::new(ChallengeCounter::TimesCompleted)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    // The total count towards this counter across all times completed
                    .col(
                        ColumnDef::new(ChallengeCounter::TotalCount)
                            .unsigned()
                            .not_null(),
                    )
                    // The current counter progress
                    .col(
                        ColumnDef::new(ChallengeCounter::CurrentCount)
                            .unsigned()
                            .not_null(),
                    )
                    // The required count for this challenge to be complete
                    .col(
                        ColumnDef::new(ChallengeCounter::TargetCount)
                            .unsigned()
                            .not_null(),
                    )
                    // The number of times this counter has been reset
                    .col(
                        ColumnDef::new(ChallengeCounter::ResetCount)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    // The last time this counter was changed
                    .col(
                        ColumnDef::new(ChallengeCounter::LastChanged)
                            .date_time()
                            .not_null(),
                    )
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChallengeCounter::Table, ChallengeCounter::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Foreign key linking for the challenge ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChallengeCounter::Table, ChallengeCounter::ChallengeId)
                            .to(ChallengeProgress::Table, ChallengeProgress::ChallengeId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Additional index for per-user per-challenge collections
        manager
            .create_index(
                Index::create()
                    .name("idx-cc-uid-cid")
                    .table(ChallengeCounter::Table)
                    .col(ChallengeCounter::UserId)
                    .col(ChallengeCounter::ChallengeId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChallengeCounter::Table).to_owned())
            .await?;
        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(ChallengeCounter::Table)
                    .name("idx-cc-uid-cid")
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ChallengeCounter {
    Table,
    UserId,
    ChallengeId,
    Name,
    TimesCompleted,
    TotalCount,
    CurrentCount,
    TargetCount,
    ResetCount,
    LastChanged,
}
