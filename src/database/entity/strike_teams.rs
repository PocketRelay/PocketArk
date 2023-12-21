use super::User;
use crate::database::DbResult;
use crate::services::character::levels::{LevelTables, ProgressionXp};
use crate::services::strike_teams::StrikeTeamEquipment;
use crate::services::strike_teams::TeamTrait;
use crate::services::Services;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{prelude::*, FromJsonQueryResult, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::{uuid, Uuid};

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "strike_teams")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: u32,
    #[serde(rename = "id")]
    pub team_id: Uuid,
    #[serde(skip)]
    pub user_id: u32,
    pub name: String,
    pub icon: StrikeTeamIcon,
    pub level: u32,
    pub xp: ProgressionXp,
    pub equipment: Option<StrikeTeamEquipment>,
    pub positive_traits: TraitList,
    pub negative_traits: TraitList,
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
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct TraitList(pub Vec<TeamTrait>);

// Sourced from "NATO phonetic alphabet"
static STRIKE_TEAM_NAMES: &[&str] = &[
    "Yankee", "Delta", "India", "Echo", "Zulu", "Charlie", "Whiskey", "Lima", "Bravo", "Sierra",
    "November", "X-Ray", "Golf", "Alpha", "Romeo", "Kilo", "Tango", "Quebec", "Foxtrot", "Papa",
    "Mike", "Oscar", "Juliet", "Uniform", "Victor", "Hotel",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamIcon {
    pub name: String,
    pub image: String,
}

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
            name: name.to_string(),
            image: image.to_string(),
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
        let services = Services::get();
        let mut rng = StdRng::from_entropy();
        let mut strike_team = Self::random(&mut rng, &services.level_tables);
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

    pub async fn get_by_id<C>(db: &C, user: &User, id: Uuid) -> DbResult<Option<Self>>
    where
        C: ConnectionTrait + Send,
    {
        user.find_related(Entity)
            .filter(Column::TeamId.eq(id))
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
            .get(&Self::LEVEL_TABLE)
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
            user_id: NotSet,
            id: NotSet,
            team_id: Set(Uuid::new_v4()),
            name: Set(name),
            icon: Set(icon),
            level: Set(level),
            xp: Set(xp),
            equipment: Set(None),
            positive_traits: Set(TraitList(positive_traits)),
            negative_traits: Set(TraitList(negative_traits)),
            out_of_date: Set(false),
        }
    }
}
