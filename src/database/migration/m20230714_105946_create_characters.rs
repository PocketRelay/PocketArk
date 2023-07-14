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
                    .table(Characters::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Characters::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Characters::CharacterId).uuid().not_null())
                    .col(ColumnDef::new(Characters::UserId).unsigned().not_null())
                    .col(ColumnDef::new(Characters::ClassName).uuid().not_null())
                    .col(ColumnDef::new(Characters::Name).uuid().not_null())
                    .col(ColumnDef::new(Characters::Level).unsigned().not_null())
                    .col(ColumnDef::new(Characters::Xp).json().not_null())
                    .col(ColumnDef::new(Characters::Promotion).unsigned().not_null())
                    .col(ColumnDef::new(Characters::Points).json().not_null())
                    .col(ColumnDef::new(Characters::PointsSpent).json().not_null())
                    .col(ColumnDef::new(Characters::PointsGranted).json().not_null())
                    .col(ColumnDef::new(Characters::SkillTrees).json().not_null())
                    .col(ColumnDef::new(Characters::Attributes).json().not_null())
                    .col(ColumnDef::new(Characters::Bonus).json().not_null())
                    .col(ColumnDef::new(Characters::Equipments).json().not_null())
                    .col(ColumnDef::new(Characters::Customization).json().not_null())
                    .col(ColumnDef::new(Characters::PlayStats).json().not_null())
                    .col(
                        ColumnDef::new(Characters::InventoryNamespace)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Characters::LastUsed).date_time().null())
                    .col(ColumnDef::new(Characters::Promotable).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Characters::Table, Characters::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Characters::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Characters {
    Table,
    Id,
    CharacterId,
    UserId,
    ClassName,
    Name,
    Level,
    Xp,
    Promotion,
    Points,
    PointsSpent,
    PointsGranted,
    SkillTrees,
    Attributes,
    Bonus,
    Equipments,
    Customization,
    PlayStats,
    InventoryNamespace,
    LastUsed,
    Promotable,
}
