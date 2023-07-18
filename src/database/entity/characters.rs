use super::{User, ValueMap};
use crate::{
    database::{entity::ClassData, DbResult},
    http::models::{
        auth::Sku,
        character::{CharacterEquipment, Class, CustomizationEntry, SkillTreeEntry, Xp},
    },
    services::defs::{Definitions, LevelTables},
    state::App,
};
use sea_orm::{entity::prelude::*, ActiveValue};
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
    pub async fn create_default(user: &User, db: &DatabaseConnection) -> DbResult<()> {
        let services: &crate::services::Services = App::services();
        let defs = &services.defs;

        // Create models from initial item defs
        let character_items = [
            "79f3511c-55da-67f0-5002-359c370015d8", // HUMAN FEMALE SOLDIER
            "a3960123-3625-4126-82e4-1f9a127d33aa", // HUMAN MALE ENGINEER
            "baae0381-8690-4097-ae6d-0c16473519b4", // HUMAN MALE SENTINEL
            "c756c741-1bc8-47a8-9f35-b7ca943ba034", // HUMAN FEMALE ENGINEER
            "e4357633-93bc-4596-99c3-4cc0a49b2277", // HUMAN MALE ADEPT
            "7fd30824-e20c-473e-b906-f4f30ebc4bb0", // HUMAN MALE VANGUARD
            "96fa16c5-9f2b-46f8-a491-a4b0a24a1089", // HUMAN FEMALE VANGUARD
            "34aeef66-a030-445e-98e2-1513c0c78df4", // HUMAN MALE INFILTRATOR
            "af3a2cf0-dff7-4ca8-9199-73ce546c3e7b", // HUMAN MALE SOLDIER
            "319ffe5d-f8fb-4217-bd2f-2e8af4f53fc8", // HUMAN FEMALE SENTINEL
            "e2f76cf1-4b42-4dba-9751-f2add5c3f654", // HUMAN FEMALE ADEPT
        ];

        for item in character_items {
            if let Ok(uuid) = Uuid::parse_str(item) {
                Self::create_from_item(defs, user, uuid, db).await?;
            }
        }

        Ok(())
    }

    pub async fn create_from_item(
        defs: &Definitions,
        user: &User,
        uuid: Uuid,
        db: &DatabaseConnection,
    ) -> DbResult<()> {
        use sea_orm::ActiveValue::{NotSet, Set};

        const DEFAULT_LEVEL: u32 = 1;
        const DEFAULT_SKILL_POINTS: u32 = 2;

        let class_def = match defs.classes.lookup(&uuid) {
            Some(value) => value,
            // Class definition for the item is missing (Item wasn't a character?)
            None => return Ok(()),
        };

        let mut point_map = HashMap::new();
        point_map.insert("MEA_skill_points".to_string(), DEFAULT_SKILL_POINTS);

        let xp = Self::xp_from_level(&defs.level_tables, class_def.level_name, DEFAULT_LEVEL);

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

        ClassData::create(user, class_def.name, true, db).await?;

        Ok(())
    }

    /// Obtains the xp structure which contains the current, next, and last
    /// xp requirements for the provided level using the level tables
    fn xp_from_level(tables: &LevelTables, level_name: Uuid, level: u32) -> Xp {
        let (current, last, next) = match tables.lookup(&level_name) {
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
