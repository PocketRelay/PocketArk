use uuid::Uuid;

use crate::{
    database::{
        DbExecutor, DbResult,
        dto::{
            character::{CharacterDto, CharacterId, CreateCharacterDto},
            users::UserId,
        },
        extensions::SqlxBindExt,
    },
    definitions::{
        classes::{CharacterEquipment, ClassName, CustomizationMap},
        level_tables::ProgressionXp,
        skills::SkillTree,
    },
    http::models::character::CharacterEquipmentList,
};

pub struct CharactersRepository;

impl CharactersRepository {
    /// Create a new character
    pub async fn create(db: impl DbExecutor<'_>, create: CreateCharacterDto) -> DbResult<()> {
        sqlx::query_as(
            r#"
            INSERT INTO "characters" (
                "character_id", "user_id", "class_name",
                "level", "xp", "promotion", "points",
                "points_spent", "points_granted", "skill_trees",
                "attributes", "bonus", "equipments", "customization",
                "play_stats"
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? ,?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(create.character_id)
        .bind(create.user_id)
        .bind(create.class_name)
        .bind(create.level)
        .bind_json(create.xp)?
        .bind(create.promotion)
        .bind_json(create.points)?
        .bind_json(create.points_spent)?
        .bind_json(create.points_granted)?
        .bind_json(create.skill_trees)?
        .bind_json(create.attributes)?
        .bind_json(create.bonus)?
        .bind_json(create.equipments)?
        .bind_json(create.customization)?
        .bind_json(create.play_stats)?
        .fetch_one(db)
        .await
    }

    /// Set the XP level for a character
    pub async fn set_xp_level(
        db: impl DbExecutor<'_>,
        id: CharacterId,
        xp: ProgressionXp,
        level: u32,
    ) -> DbResult<bool> {
        let result = sqlx::query(r#"UPDATE "characters" SET "xp" = ?, "level" = ? WHERE "id" = ?"#)
            .bind_json(xp)?
            .bind(level)
            .bind(id)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update the customization data for a character
    pub async fn set_customization(
        db: impl DbExecutor<'_>,
        id: CharacterId,
        customization: CustomizationMap,
    ) -> DbResult<bool> {
        let result = sqlx::query(r#"UPDATE "characters" SET "customization" = ? WHERE "id" = ?"#)
            .bind_json(customization)?
            .bind(id)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update the equipment data for a character
    pub async fn set_equipments(
        db: impl DbExecutor<'_>,
        id: CharacterId,
        equipments: Vec<CharacterEquipment>,
    ) -> DbResult<bool> {
        let result = sqlx::query(r#"UPDATE "characters" SET "equipments" = ? WHERE "id" = ?"#)
            .bind_json(equipments)?
            .bind(id)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
    /// Update the equipment data for a character
    pub async fn set_skill_trees(
        db: impl DbExecutor<'_>,
        id: CharacterId,
        skill_trees: Vec<SkillTree>,
    ) -> DbResult<bool> {
        let result = sqlx::query(r#"UPDATE "characters" SET "skill_trees" = ? WHERE "id" = ?"#)
            .bind_json(skill_trees)?
            .bind(id)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the character for a user by its game level character ID
    pub async fn get_by_user_by_id(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        character_id: Uuid,
    ) -> DbResult<Option<CharacterDto>> {
        sqlx::query_as(r#"SELECT * FROM "characters" WHERE "user_id" = ? AND "character_id" = ?"#)
            .bind(user_id)
            .bind(character_id)
            .fetch_optional(db)
            .await
    }

    /// Get the characters for the user
    pub async fn get_by_user(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<CharacterDto>> {
        sqlx::query_as(r#"SELECT * FROM "characters" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_all(db)
            .await
    }

    /// Get the character for a user by its class name
    pub async fn get_by_user_by_class(
        db: impl DbExecutor<'_>,
        user_id: UserId,
        class_name: ClassName,
    ) -> DbResult<Option<CharacterDto>> {
        sqlx::query_as(r#"SELECT * FROM "characters" WHERE "user_id" = ? AND "class_name" = ?"#)
            .bind(user_id)
            .bind(class_name)
            .fetch_optional(db)
            .await
    }

    /// Get the class names of all the users characters
    pub async fn get_user_classes(
        db: impl DbExecutor<'_>,
        user_id: UserId,
    ) -> DbResult<Vec<ClassName>> {
        sqlx::query_scalar(r#"SELECT "class_name" FROM "characters" WHERE "user_id" = ?"#)
            .bind(user_id)
            .fetch_all(db)
            .await
    }
}
