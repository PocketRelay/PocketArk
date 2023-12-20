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
                    // Unqiue ID for this character
                    .col(
                        ColumnDef::new(Characters::Id)
                            .unsigned()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    // ID of the user this character belongs to
                    .col(ColumnDef::new(Characters::UserId).unsigned().not_null())
                    // UUID of the character class definition
                    .col(ColumnDef::new(Characters::ClassName).uuid().not_null())
                    // Character level number
                    .col(ColumnDef::new(Characters::Level).unsigned().not_null())
                    // Character XP data (Current -> Next -> Last)
                    .col(ColumnDef::new(Characters::Xp).json().not_null())
                    // Character promotion level
                    .col(ColumnDef::new(Characters::Promotion).unsigned().not_null())
                    // Mapping for different kind of points
                    .col(ColumnDef::new(Characters::Points).json().not_null())
                    // Mapping for different kind of points that are spent
                    .col(ColumnDef::new(Characters::PointsSpent).json().not_null())
                    // Mapping for different kind of points that were granted at some point
                    .col(ColumnDef::new(Characters::PointsGranted).json().not_null())
                    // Skill tree selection data
                    .col(ColumnDef::new(Characters::SkillTrees).json().not_null())
                    // Additional character attributes
                    .col(ColumnDef::new(Characters::Attributes).json().not_null())
                    // Bonus data map
                    .col(ColumnDef::new(Characters::Bonus).json().not_null())
                    // Character equipment list
                    .col(ColumnDef::new(Characters::Equipments).json().not_null())
                    // Character customization data (Map)
                    .col(ColumnDef::new(Characters::Customization).json().not_null())
                    // Character usage stats
                    .col(ColumnDef::new(Characters::PlayStats).json().not_null())
                    // Last time the character was used
                    .col(ColumnDef::new(Characters::LastUsed).date_time().null())
                    // Whether the character is promotable
                    .col(ColumnDef::new(Characters::Promotable).boolean().not_null())
                    // Foreign key linking for the User ID
                    .foreign_key(
                        ForeignKey::create()
                            .from(Characters::Table, Characters::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create a unique index accross the user ID and character class name
        // (Users should only have a single character per class name)
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx-character-uid-def")
                    .table(Characters::Table)
                    .col(Characters::UserId)
                    .col(Characters::ClassName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the table
        manager
            .drop_table(Table::drop().table(Characters::Table).to_owned())
            .await?;

        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .table(Characters::Table)
                    .name("idx-character-uid-def")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Characters {
    Table,
    Id,
    UserId,
    ClassName,
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
    LastUsed,
    Promotable,
}
