use super::{User, ValueMap};
use crate::{
    database::DbResult,
    http::models::{
        auth::Sku,
        character::{CharacterEquipment, CustomizationEntry, SkillTreeEntry, Xp},
    },
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
                Self::create_from_item(user, uuid, db).await?;
            }
        }

        Ok(())
    }

    pub async fn create_from_item(
        user: &User,
        uuid: Uuid,
        db: &DatabaseConnection,
    ) -> DbResult<()> {
        let services = App::services();
        let defs = &services.defs;
        let class_def = defs.classes.map.get(&uuid);
        if let Some(class_def) = class_def {
            let mut point_map = HashMap::new();
            point_map.insert("MEA_skill_points".to_string(), 2);

            let model = ActiveModel {
                id: ActiveValue::NotSet,
                character_id: ActiveValue::Set(Uuid::new_v4()),
                user_id: ActiveValue::Set(user.id),
                class_name: ActiveValue::Set(class_def.name),
                name: ActiveValue::Set(class_def.name),
                level: ActiveValue::Set(1),
                xp: ActiveValue::Set(Xp {
                    current: 0,
                    last: 0,
                    next: 11000,
                }),
                promotion: ActiveValue::Set(0),
                points: ActiveValue::Set(PointMap(point_map)),
                points_spent: ActiveValue::Set(PointMap::default()),
                points_granted: ActiveValue::Set(PointMap::default()),
                skill_trees: ActiveValue::Set(SkillTree(class_def.skill_trees.clone())),
                attributes: ActiveValue::Set(ValueMap(class_def.attributes.clone())),
                bonus: ActiveValue::Set(ValueMap(class_def.bonus.clone())),
                equipments: ActiveValue::Set(EquipmentList(class_def.default_equipments.clone())),
                customization: ActiveValue::Set(CustomizationMap(
                    class_def.default_customization.clone(),
                )),
                play_stats: ActiveValue::Set(ValueMap::default()),
                inventory_namespace: ActiveValue::Set(class_def.default_namespace.clone()),
                last_used: ActiveValue::Set(None),
                promotable: ActiveValue::Set(false),
            };
            let _ = model.insert(db).await?;
        }
        Ok(())
    }
}
