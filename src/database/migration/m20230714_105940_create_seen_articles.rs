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
                    .table(SeenArticle::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeenArticle::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SeenArticle::UserId).unsigned().not_null())
                    .col(ColumnDef::new(SeenArticle::ArticleId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(SeenArticle::Table, SeenArticle::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeenArticle::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum SeenArticle {
    Table,
    Id,
    UserId,
    ArticleId,
}
