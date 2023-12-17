use super::{User, ValueMap};
use crate::{
    database::{entity::ClassData, DbResult},
    services::{
        character::{CharacterEquipment, CharacterService, CustomizationEntry, SkillTreeEntry, Xp},
        items::ItemName,
    },
    utils::models::Sku,
};
use sea_orm::{
    entity::prelude::*,
    ActiveValue::{NotSet, Set},
    FromJsonQueryResult, IntoActiveModel,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
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
    pub class_name: Uuid,
    /// The current level of the characters
    pub level: u32,
    /// XP progression data associated with this character
    pub xp: Xp,
    /// Number of promotions this character has been given
    pub promotion: u32,
    /// Mapping for available point allocations
    pub points: PointMap,
    /// Mapping for spent point allocations
    pub points_spent: PointMap,
    /// Mapping for total points given
    pub points_granted: PointMap,
    /// Skill tree progression data
    pub skill_trees: SkillTree,
    /// Character attributes
    pub attributes: ValueMap,
    /// Character bonus data
    pub bonus: ValueMap,
    /// Character equipment list
    pub equipments: EquipmentList,
    /// Character customization data
    pub customization: CustomizationMap,
    /// Character usage stats
    pub play_stats: PlayStats,
    /// Last time this character was used
    pub last_used: Option<DateTimeUtc>,
    /// Whether this chracter is promotable
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct PointMap(pub HashMap<String, u32>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct CustomizationMap(pub HashMap<String, CustomizationEntry>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct EquipmentList(pub Vec<CharacterEquipment>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SkillTree(pub Vec<SkillTreeEntry>);

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
    pub fn update_xp<C>(
        self,
        db: &C,
        xp: Xp,
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

    pub async fn create_from_item<C>(
        db: &C,
        characters: &CharacterService,
        user: &User,
        item_name: &ItemName,
    ) -> DbResult<()>
    where
        C: ConnectionTrait + Send,
    {
        const DEFAULT_LEVEL: u32 = 1;
        const DEFAULT_SKILL_POINTS: u32 = 2;

        let class_def = match characters.classes.by_item(item_name) {
            Some(value) => value,
            // Class definition for the item is missing (Item wasn't a character?)
            None => return Ok(()),
        };

        let mut point_map = HashMap::new();
        point_map.insert("MEA_skill_points".to_string(), DEFAULT_SKILL_POINTS);

        let xp = characters.xp_from_level(class_def.level_name, DEFAULT_LEVEL);

        // Insert character
        let model = ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            class_name: Set(class_def.name),
            level: Set(DEFAULT_LEVEL),
            xp: Set(xp),
            promotion: Set(0),
            points: Set(PointMap(point_map)),
            points_spent: Set(PointMap::default()),
            points_granted: Set(PointMap::default()),
            skill_trees: Set(SkillTree(class_def.skill_trees.clone())),
            attributes: Set(ValueMap(class_def.attributes.clone())),
            bonus: Set(ValueMap(class_def.bonus.clone())),
            equipments: Set(EquipmentList(class_def.default_equipments.clone())),
            customization: Set(CustomizationMap(class_def.default_customization.clone())),
            play_stats: Set(PlayStats::default()),
            last_used: Set(None),
            promotable: Set(false),
        };
        let _ = model.insert(db).await?;

        ClassData::create(db, user, class_def.name, true).await?;

        Ok(())
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
}

/// Serialization implementation
impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state: <S as Serializer>::SerializeStruct =
            Serializer::serialize_struct(serializer, "Character", 19)?;
        state.serialize_field("characterId", &self.id.to_string());
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
