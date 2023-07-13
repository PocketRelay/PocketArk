use sea_orm_migration::prelude::*;

use super::{m20230714_105755_create_users::Users, m20230714_113455_class_data::ClassData};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Character::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Character::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Character::UserId).unsigned().not_null())
                    .col(ColumnDef::new(Character::ClassName).uuid().not_null())
                    .col(ColumnDef::new(Character::Name).uuid().not_null())
                    .col(ColumnDef::new(Character::Level).unsigned().not_null())
                    .col(ColumnDef::new(Character::Xp).json().not_null())
                    .col(ColumnDef::new(Character::Promotion).unsigned().not_null())
                    .col(ColumnDef::new(Character::Points).json().not_null())
                    .col(ColumnDef::new(Character::PointsSpent).json().not_null())
                    .col(ColumnDef::new(Character::PointsGranted).json().not_null())
                    .col(ColumnDef::new(Character::SkillTrees).json().not_null())
                    .col(ColumnDef::new(Character::Attributes).json().not_null())
                    .col(ColumnDef::new(Character::Bonus).json().not_null())
                    .col(ColumnDef::new(Character::Equipments).json().not_null())
                    .col(ColumnDef::new(Character::Customization).json().not_null())
                    .col(ColumnDef::new(Character::PlayStats).json().not_null())
                    .col(
                        ColumnDef::new(Character::InventoryNamespace)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Character::LastUsed).date_time().null())
                    .col(ColumnDef::new(Character::Promotable).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Character::Table, Character::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Character::Table, Character::ClassName)
                            .to(ClassData::Table, ClassData::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Character::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Character {
    Table,
    Id,
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
