use super::m20230714_105755_create_users::Users;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the currency data table
        manager
            .create_table(
                Table::create()
                    .table(Currency::Table)
                    .if_not_exists()
                    // This table uses a composite key over the UserId and currency type
                    .primary_key(Index::create().col(Currency::UserId).col(Currency::Ty))
                    // ID of the user this currency data belongs to
                    .col(ColumnDef::new(Currency::UserId).unsigned().not_null())
                    // The ty of the currency
                    .col(ColumnDef::new(Currency::Ty).string().not_null())
                    // The amount of currency the user has
                    .col(ColumnDef::new(Currency::Balance).big_integer().not_null())
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(Currency::Table, Currency::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Additional index for per-user currency data collections
        manager
            .create_index(
                Index::create()
                    .table(Currency::Table)
                    .name("idx-currency-uid")
                    .col(Currency::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Currency::Table).to_owned())
            .await?;

        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(Currency::Table)
                    .name("idx-currency-uid")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Currency {
    Table,
    Ty,
    UserId,
    Balance,
}
