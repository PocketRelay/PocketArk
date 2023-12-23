use super::{characters::CharacterId, User};
use crate::{
    database::DbResult,
    definitions::{
        classes::CharacterEquipment,
        i18n::{I18nDescription, I18nName},
        level_tables::ProgressionXp,
    },
};
use sea_orm::{entity::prelude::*, ActiveValue::Set, FromJsonQueryResult, IntoActiveModel};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

/// Shared data database structure
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "shared_data")]
pub struct Model {
    /// The ID of the user this data belongs to
    #[sea_orm(primary_key)]
    pub user_id: u32,
    // ID of the currently active character for the user
    pub active_character_id: Option<CharacterId>,
    // Shared statistis about the user
    pub shared_stats: SharedStats,
    // Shared equipment configuration
    pub shared_equipment: CharacterSharedEquipment,
    // Shared progression states
    pub shared_progression: SharedProgressionList,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct SharedStats {
    /// The pathfinder rating for the user
    pub pathfinder_rating: f32,
    /// Other shared stats
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SharedProgressionList(pub Vec<SharedProgression>);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct CharacterSharedEquipment {
    pub list: Vec<CharacterEquipment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SharedProgression {
    pub name: Uuid,
    #[serde(flatten)]
    pub i18n_name: I18nName,
    #[serde(flatten)]
    pub i18n_description: I18nDescription,
    pub level: u32,
    pub xp: ProgressionXp,
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

impl Model {
    pub fn create_default<'db, C>(
        db: &'db C,
        user: &User,
    ) -> impl Future<Output = DbResult<Model>> + 'db
    where
        C: ConnectionTrait + Send,
    {
        ActiveModel {
            user_id: Set(user.id),
            active_character_id: Set(None),
            shared_equipment: Set(Default::default()),
            shared_progression: Set(Default::default()),
            shared_stats: Set(Default::default()),
        }
        .insert(db)
    }

    /// Loads the shared data for the provided `user`, will create
    /// new shared data if they don't have one
    pub async fn get<C>(db: &C, user: &User) -> DbResult<Model>
    where
        C: ConnectionTrait + Send,
    {
        // User already has shared data defined
        if let Some(shared_data) = user.find_related(Entity).one(db).await? {
            return Ok(shared_data);
        }

        // Create new default shared data
        Self::create_default(db, user).await
    }

    pub fn set_shared_equipment<C>(
        self,
        db: &C,
        list: Vec<CharacterEquipment>,
    ) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut shared_data = self.into_active_model();
        shared_data.shared_equipment = Set(CharacterSharedEquipment { list });
        shared_data.update(db)
    }

    pub fn set_active_character<C>(
        self,
        db: &C,
        character_id: CharacterId,
    ) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut shared_data = self.into_active_model();
        shared_data.active_character_id = Set(Some(character_id));
        shared_data.update(db)
    }

    pub fn save_progression<C>(self, db: &C) -> impl Future<Output = DbResult<Self>> + '_
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.shared_progression = Set(model
            .shared_progression
            .take()
            .expect("Shared progression missing from take"));
        model.update(db)
    }
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = serializer.serialize_struct("SharedData", 4)?;
        value.serialize_field(
            "activeCharacterId",
            &self.active_character_id.map(|value| value.to_string()),
        )?;
        value.serialize_field("sharedStats", &self.shared_stats)?;
        value.serialize_field("sharedEquipment", &self.shared_equipment)?;
        value.serialize_field("sharedProgression", &self.shared_progression)?;
        value.end()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
