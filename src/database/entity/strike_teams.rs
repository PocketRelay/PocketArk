use super::users::UserId;
use super::{SeaJson, StrikeTeamMissionProgress, User};
use crate::database::DbResult;
use crate::definitions::{
    level_tables::{LevelTables, ProgressionXp},
    striketeams::{StrikeTeamEquipment, TeamTrait},
};
use crate::utils::ImStr;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use sea_orm::ActiveValue::Set;
use sea_orm::{prelude::*, FromJsonQueryResult, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::{uuid, Uuid};

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
    pub name: String,
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

// Sourced from "NATO phonetic alphabet", these are the default strike team names used by the game
static STRIKE_TEAM_NAMES: &[&str] = &[
    "Yankee", "Delta", "India", "Echo", "Zulu", "Charlie", "Whiskey", "Lima", "Bravo", "Sierra",
    "November", "X-Ray", "Golf", "Alpha", "Romeo", "Kilo", "Tango", "Quebec", "Foxtrot", "Papa",
    "Mike", "Oscar", "Juliet", "Uniform", "Victor", "Hotel",
];

/// Icon that the strike team should use
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamIcon {
    /// Name of the icon
    pub name: ImStr,
    /// Icon image path
    pub image: ImStr,
}

/// Set of default known icons from the game
static ICON_SETS: &[(&str, &str)] = &[
    ("icon1", "Team01"),
    ("icon2", "Team02"),
    ("icon3", "Team03"),
    ("icon4", "Team04"),
    ("icon5", "Team05"),
    ("icon6", "Team06"),
    ("icon7", "Team07"),
    ("icon8", "Team08"),
    ("icon9", "Team09"),
    ("icon10", "Team10"),
];

impl StrikeTeamIcon {
    pub fn random(rng: &mut StdRng) -> Self {
        let (name, image) = ICON_SETS.choose(rng).expect("Missing strike team icon set");

        StrikeTeamIcon {
            name: Box::from(*name),
            image: Box::from(*image),
        }
    }
}

impl Model {
    /// The level table used for strike team levels
    const LEVEL_TABLE: Uuid = uuid!("5e6f7542-7309-9367-8437-fe83678e5c28");

    pub async fn create_default<C>(db: &C, user: &User) -> DbResult<Self>
    where
        C: ConnectionTrait + Send,
    {
        let level_tables = LevelTables::get();

        let mut rng = StdRng::from_entropy();
        let mut strike_team = Self::random(&mut rng, level_tables);
        strike_team.user_id = Set(user.id);
        strike_team.insert(db).await
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

    pub fn random(rng: &mut StdRng, level_tables: &LevelTables) -> ActiveModel {
        let name = STRIKE_TEAM_NAMES
            .choose(rng)
            .expect("Failed to choose strike team name")
            .to_string();
        let level_table = level_tables
            .by_name(&Self::LEVEL_TABLE)
            .expect("Missing strike team level table");

        let level = 1;
        let next_xp = level_table
            .get_xp_requirement(level)
            .expect("Missing xp requirement for next strike team level");

        let xp = ProgressionXp {
            current: 0,
            last: 0,
            next: next_xp,
        };

        let icon = StrikeTeamIcon::random(rng);

        let positive_traits = Vec::new();
        let negative_traits = Vec::new();

        ActiveModel {
            name: Set(name),
            icon: Set(icon),
            level: Set(level),
            xp: Set(xp),
            positive_traits: Set(SeaJson(positive_traits)),
            negative_traits: Set(SeaJson(negative_traits)),
            ..Default::default()
        }
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
