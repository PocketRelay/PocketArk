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
                    .table(ClassData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClassData::Id)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ClassData::UserId).unsigned().not_null())
                    .col(ColumnDef::new(ClassData::Name).uuid().not_null())
                    .col(ColumnDef::new(ClassData::Unlocked).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ClassData::Table, ClassData::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create a unique index accross the user ID and class name
        // (Users should only have a single item per definition)
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx-class-data-uid-name")
                    .table(ClassData::Table)
                    .col(ClassData::UserId)
                    .col(ClassData::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClassData::Table).to_owned())
            .await?;

        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(ClassData::Table)
                    .name("idx-class-data-uid-name")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum ClassData {
    Table,
    Id,
    UserId,
    Name,
    Unlocked,
}
