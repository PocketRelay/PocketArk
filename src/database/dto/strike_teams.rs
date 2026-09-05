use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sqlx::prelude::FromRow;

use crate::{
    database::dto::users::UserId,
    definitions::{
        i18n::Localized,
        level_tables::ProgressionXp,
        strike_teams::{
            equipment::StrikeTeamEquipment, icon::StrikeTeamIcon, name::StrikeTeamName,
            traits::StrikeTeamTrait,
        },
    },
};

pub type StrikeTeamId = u32;

#[derive(Debug)]
pub struct CreateStrikeTeamDto {
    pub user_id: UserId,
    pub name: StrikeTeamName,
    pub icon: StrikeTeamIcon,
    pub level: u32,
    pub xp: ProgressionXp,
    pub positive_traits: Vec<StrikeTeamTrait>,
    pub negative_traits: Vec<StrikeTeamTrait>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, FromRow, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeTeamDto {
    /// Unique ID of the strike team
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub id: StrikeTeamId,
    /// ID of the user that owns this strike team
    #[serde(skip)]
    pub user_id: UserId,
    /// Name of the strike team (Shown in game)
    pub name: StrikeTeamName,
    /// Icon to use with the strike team
    #[sqlx(json)]
    pub icon: StrikeTeamIcon,
    /// Current level of the strike team
    pub level: u32,
    /// XP progression for the strike team
    #[sqlx(json)]
    pub xp: ProgressionXp,
    /// Equipment if the strike team has one active
    #[sqlx(json)]
    pub equipment: Option<StrikeTeamEquipment>,
    /// Positive traits this strike team has
    #[sqlx(json)]
    pub positive_traits: Vec<StrikeTeamTrait>,
    /// Negative traits this strike team has
    #[sqlx(json)]
    pub negative_traits: Vec<StrikeTeamTrait>,
    /// Unknown usage
    pub out_of_date: bool,
}

impl Localized for StrikeTeamDto {
    fn localize(&mut self, i18n: &crate::definitions::i18n::I18n) {
        self.equipment.localize(i18n);
        self.positive_traits.localize(i18n);
        self.negative_traits.localize(i18n);
    }
}
