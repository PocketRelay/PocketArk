use sea_orm_migration::prelude::*;

use super::m20230714_105755_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the class data table
        manager
            .create_table(
                Table::create()
                    .table(ClassData::Table)
                    .if_not_exists()
                    // This table uses a composite key over the UserId and ClassName
                    .primary_key(
                        Index::create()
                            .col(ClassData::UserId)
                            .col(ClassData::ClassName),
                    )
                    // ID of the user this data belongs to
                    .col(ColumnDef::new(ClassData::UserId).unsigned().not_null())
                    // Name of the class definition this data is for
                    .col(ColumnDef::new(ClassData::ClassName).uuid().not_null())
                    // Whether this class is unlocked
                    .col(ColumnDef::new(ClassData::Unlocked).boolean().not_null())
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(ClassData::Table, ClassData::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Additional index for per-user class data collections
        manager
            .create_index(
                Index::create()
                    .name("idx-class-data-uid")
                    .table(ClassData::Table)
                    .col(ClassData::UserId)
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
                    .name("idx-class-data-uid")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum ClassData {
    Table,
    Id,
    UserId,
    ClassName,
    Unlocked,
}
