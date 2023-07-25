use super::{User, ValueMap};
use crate::{
    database::{entity::ClassData, DbResult},
    services::character::{
        CharacterEquipment, CharacterService, CustomizationEntry, SkillTreeEntry, Xp,
    },
    utils::models::Sku,
};
use sea_orm::{
    entity::prelude::*,
    ActiveValue::{NotSet, Set},
    IntoActiveModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "characters")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    pub character_id: Uuid,
    #[sea_orm(ignore)]
    pub sku: Sku,
    #[serde(skip)]
    pub user_id: u32,
    #[serde(rename = "characterClassName")]
    pub class_name: Uuid,
    pub name: Uuid,
    pub level: u32,
    pub xp: Xp,
    pub promotion: u32,
    pub points: PointMap,
    pub points_spent: PointMap,
    pub points_granted: PointMap,
    pub skill_trees: SkillTree,
    pub attributes: ValueMap,
    pub bonus: ValueMap,
    pub equipments: EquipmentList,
    pub customization: CustomizationMap,
    pub play_stats: ValueMap,
    pub inventory_namespace: String,
    pub last_used: Option<DateTimeUtc>,
    pub promotable: bool,
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
    pub async fn update_xp<C>(self, db: &C, xp: Xp, level: u32) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.xp = Set(xp);
        model.level = Set(level);
        model.update(db).await
    }

    pub async fn create_from_item<C>(
        db: &C,
        characters: &CharacterService,
        user: &User,
        item_name: &Uuid,
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

        let xp = Self::xp_from_level(characters, class_def.level_name, DEFAULT_LEVEL);

        // Insert character
        let model = ActiveModel {
            id: NotSet,
            character_id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            class_name: Set(class_def.name),
            name: Set(class_def.name),
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
            play_stats: Set(ValueMap::default()),
            inventory_namespace: Set(class_def.default_namespace.clone()),
            last_used: Set(None),
            promotable: Set(false),
        };
        let _ = model.insert(db).await?;

        ClassData::create(db, user, class_def.name, true).await?;

        Ok(())
    }

    /// Obtains the xp structure which contains the current, next, and last
    /// xp requirements for the provided level using the level tables
    fn xp_from_level(characters: &CharacterService, level_name: Uuid, level: u32) -> Xp {
        let (current, last, next) = match characters.level_table(&level_name) {
            Some(table) => {
                let current = table.get_entry_xp(level).unwrap_or(0);
                let last = table.get_entry_xp(level - 1).unwrap_or(0);
                let next = table.get_entry_xp(level + 1).unwrap_or(0);
                (current, last, next)
            }
            // Empty values when level table is unknwon
            None => (0, 0, 0),
        };

        Xp {
            current,
            next,
            last,
        }
    }

    pub async fn find_by_id_user(
        db: &DatabaseConnection,
        user: &User,
        id: Uuid,
    ) -> DbResult<Option<Self>> {
        user.find_related(Entity)
            .filter(Column::CharacterId.eq(id))
            .one(db)
            .await
    }
}
