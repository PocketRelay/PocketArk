use super::users::UserId;
use super::{SeaJson, StrikeTeamMissionProgress, User};
use crate::database::DbResult;
use crate::definitions::strike_teams::{StrikeTeamData, StrikeTeamIcon, StrikeTeamName};
use crate::definitions::{
    level_tables::ProgressionXp,
    striketeams::{StrikeTeamEquipment, TeamTrait},
};
use sea_orm::ActiveValue::Set;
use sea_orm::{prelude::*, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Strike Team ID keying has been replaced with integer keys rather than the UUIDs
/// used by the official game, this is because its *very* annoying to work with
/// UUIDs as primary keys in the SQLite database (Basically defeats the purpose of SeaORM)
pub type StrikeTeamId = u32;

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "strike_teams")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Unique ID of the strike team
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: StrikeTeamId,
    /// ID of the user that owns this strike team
    #[serde(skip)]
    pub user_id: UserId,
    /// Name of the strike team (Shown in game)
    pub name: StrikeTeamName,
    /// Icon to use with the strike team
    pub icon: StrikeTeamIcon,
    /// Current level of the strike team
    pub level: u32,
    /// XP progression for the strike team
    pub xp: ProgressionXp,
    /// Equipment if the strike team has one active
    pub equipment: Option<StrikeTeamEquipment>,
    /// Positive traits this strike team has
    pub positive_traits: SeaJson<Vec<TeamTrait>>,
    /// Negative traits this strike team has
    pub negative_traits: SeaJson<Vec<TeamTrait>>,
    /// Unknown usage
    pub out_of_date: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,

    #[sea_orm(has_one = "super::strike_team_mission_progress::Entity")]
    MissionProgress,
}

impl Model {
    pub async fn create<C>(db: &C, user: &User, data: StrikeTeamData) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        ActiveModel {
            user_id: Set(user.id),
            name: Set(data.name),
            icon: Set(data.icon),
            level: Set(data.level),
            xp: Set(data.xp),
            positive_traits: Set(SeaJson(vec![data.positive_trait])),
            negative_traits: Set(Default::default()),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    pub async fn set_equipment<C>(
        self,
        db: &C,
        equipment: Option<StrikeTeamEquipment>,
    ) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        let mut model = self.into_active_model();
        model.equipment = Set(equipment);
        model.update(db).await
    }

    pub async fn delete<C>(self, db: &C) -> DbResult<()>
    where
        C: ConnectionTrait + Send,
    {
        <Self as ModelTrait>::delete(self, db).await?;
        Ok(())
    }

    // Checks if the strike team is on a mission
    pub async fn is_on_mission<C>(&self, db: &C) -> DbResult<bool>
    where
        C: ConnectionTrait + Send,
    {
        StrikeTeamMissionProgress::get_by_team(db, self)
            .await
            .map(|value| value.is_some())
    }

    pub async fn get_by_id<C>(db: &C, user: &User, id: StrikeTeamId) -> DbResult<Option<Self>>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::Id.eq(id))
            .one(db)
            .await
    }

    pub async fn get_by_user<C>(db: &C, user: &User) -> DbResult<Vec<Self>>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).all(db).await
    }

    pub async fn get_user_count<C>(db: &C, user: &User) -> DbResult<u64>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity).count(db).await
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::strike_team_mission_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MissionProgress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
