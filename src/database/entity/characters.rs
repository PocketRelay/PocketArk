use super::{SeaGenericMap, SeaJson, User, users::UserId};
use crate::{
    database::DbResult,
    definitions::{
        classes::{
            CharacterAttributes, CharacterBonus, CharacterEquipment, ClassName, CustomizationMap,
            PointMap,
        },
        level_tables::ProgressionXp,
        skills::SkillTree,
    },
    utils::models::Sku,
};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    FromJsonQueryResult, IntoActiveModel, QuerySelect,
    entity::prelude::*,
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, future::Future};
use uuid::Uuid;

/// Character ID keying has been replaced with integer keys rather than the UUIDs
/// used by the official game, this is because its *very* annoying to work with
/// UUIDs as primary keys in the SQLite database (Basically defeats the purpose of SeaORM)
pub type CharacterId = u32;

/// Character data database structure
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "characters")]
pub struct Model {
    /// Unique ID of the character
    #[sea_orm(primary_key)]
    pub id: CharacterId,
    /// ID of the user that owns this character
    pub user_id: u32,
    /// Name of the class definition this character belongs to
    pub class_name: ClassName,
    /// The current level of the characters
    pub level: u32,
    /// XP progression data associated with this character
    pub xp: ProgressionXp,
    /// Number of promotions this character has been given
    pub promotion: u32,
    /// Mapping for available point allocations
    pub points: PointMap,
    /// Mapping for spent point allocations
    pub points_spent: PointMap,
    /// Mapping for total points given
    pub points_granted: PointMap,
    /// Skill tree progression data
    pub skill_trees: SeaJson<Vec<SkillTree>>,
    /// Character attributes
    pub attributes: CharacterAttributes,
    /// Character bonus data
    pub bonus: SeaGenericMap,
    /// Character equipment list
    pub equipments: SeaJson<Vec<CharacterEquipment>>,
    /// Character customization data
    pub customization: CustomizationMap,
    /// Character usage stats
    pub play_stats: PlayStats,
    /// Last time this character was used
    pub last_used: Option<DateTimeUtc>,
    /// Whether this character is promotable
    pub promotable: bool,
}

/// TODO: Ensure this structure is complete
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PlayStats {
    pub career_duration: Option<f32>,
    /// Catch-all for unknown keys that haven't been determined yet
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    #[allow(unused)]
    pub fn update_xp<C>(
        self,
        db: &C,
        xp: ProgressionXp,
        level: u32,
    ) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.xp = Set(xp);
        model.level = Set(level);
        model.update(db)
    }

    pub fn update_customization<C>(
        self,
        db: &C,
        customization: CustomizationMap,
    ) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.customization = Set(customization);
        model.update(db)
    }

    /// Creates a new character from the provided base details
    #[allow(clippy::too_many_arguments)]
    pub fn create<'db, C>(
        db: &'db C,
        user: &User,
        class_name: ClassName,
        level: u32,
        xp: ProgressionXp,
        points: PointMap,
        skill_trees: Vec<SkillTree>,
        attributes: CharacterAttributes,
        bonus: CharacterBonus,
        equipment: Vec<CharacterEquipment>,
        customization: CustomizationMap,
    ) -> impl Future<Output = DbResult<Self>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            class_name: Set(class_name),
            level: Set(level),
            xp: Set(xp),
            promotion: Set(0),
            points: Set(points),
            // 3 of the 5 points are spent by default
            points_spent: Set(PointMap {
                skill_points: Some(3),
            }),
            points_granted: Set(PointMap::default()),
            skill_trees: Set(SeaJson(skill_trees)),
            attributes: Set(attributes),
            bonus: Set(SeaJson(bonus)),
            equipments: Set(SeaJson(equipment)),
            customization: Set(customization),
            play_stats: Set(PlayStats::default()),
            last_used: Set(None),
            promotable: Set(false),
        }
        .insert(db)
    }

    pub fn find_by_id_user<'db, C>(
        db: &'db C,
        user: &User,
        id: CharacterId,
    ) -> impl Future<Output = DbResult<Option<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).filter(Column::Id.eq(id)).one(db)
    }

    pub fn find_by_user_by_def<'db, C>(
        db: &'db C,
        user: &User,
        class_name: Uuid,
    ) -> impl Future<Output = DbResult<Option<Self>>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::ClassName.eq(class_name))
            .one(db)
    }

    /// Collects all the [ClassName]s of the classes that the provided user
    /// has unlocked
    pub async fn get_user_classes<C>(db: &C, user: &User) -> DbResult<Vec<ClassName>>
    where
        C: ConnectionTrait + Send,
    {
        let values: Vec<(UserId, ClassName)> = Entity::find()
            .select_only()
            .column(Column::UserId)
            .column(Column::ClassName)
            .filter(Column::UserId.eq(user.id))
            .into_tuple()
            .all(db)
            .await?;

        Ok(values
            .into_iter()
            .map(|(_, class_name)| class_name)
            .collect())
    }
}

/// Serialization implementation
impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state: <S as Serializer>::SerializeStruct =
            Serializer::serialize_struct(serializer, "Character", 19)?;
        state.serialize_field("characterId", &self.id.to_string())?;
        state.serialize_field("sku", &Sku)?;
        state.serialize_field("characterClassName", &self.class_name)?;
        state.serialize_field("name", &self.class_name)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("xp", &self.xp)?;
        state.serialize_field("promotion", &self.promotion)?;
        state.serialize_field("points", &self.points)?;
        state.serialize_field("pointsSpent", &self.points_spent)?;
        state.serialize_field("pointsGranted", &self.points_granted)?;
        state.serialize_field("skillTrees", &self.skill_trees)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("bonus", &self.bonus)?;
        state.serialize_field("equipments", &self.equipments)?;
        state.serialize_field("customization", &self.customization)?;
        state.serialize_field("playStats", &self.play_stats)?;
        // Inventory namespace always appears to be "default"
        state.serialize_field("inventoryNamespace", "default")?;
        state.serialize_field("lastUsed", &self.last_used)?;
        state.serialize_field("promotable", &self.promotable)?;
        state.end()
    }
}
