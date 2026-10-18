use anyhow::Context;
use pocket_ark_definitions::{
    items::{ItemDefinition, Items},
    strike_teams::{
        StrikeTeams,
        equipment::StrikeTeamEquipmentList,
        mission::{MissionDefinitions, tag::MissionTags},
        specialization::StrikeTeamSpecializations,
        traits::StrikeTeamTraits,
    },
};

use crate::resources::Resources;

pub async fn initialize_strike_teams() -> anyhow::Result<()> {
    const EQUIPMENT_RESOURCE: &str = "strikeTeam/strikeTeamEquipment.json";
    const MISSIONS_RESOURCE: &str = "strikeTeam/strikeTeamMissions.json";
    const SPECIALIZATION_RESOURCE: &str = "strikeTeam/strikeTeamSpecialization.json";
    const TAGS_RESOURCE: &str = "strikeTeam/strikeTeamTags.json";
    const TRAITS_RESOURCE: &str = "strikeTeam/strikeTeamTraits.json";

    let equipment = Resources::get_resource_bytes(EQUIPMENT_RESOURCE).await?;
    let missions = Resources::get_resource_bytes(MISSIONS_RESOURCE).await?;
    let specializations = Resources::get_resource_bytes(SPECIALIZATION_RESOURCE).await?;
    let tags = Resources::get_resource_bytes(TAGS_RESOURCE).await?;
    let traits = Resources::get_resource_bytes(TRAITS_RESOURCE).await?;

    let equipment = StrikeTeamEquipmentList::from_bytes(&equipment)
        .context("failed to parse strike team equipment")?;
    let missions = MissionDefinitions::from_bytes(&missions)
        .context("failed to parse strike team mission definitions")?;
    let specializations = StrikeTeamSpecializations::from_bytes(&specializations)
        .context("failed to parse strike team specialization definitions")?;
    let tags = MissionTags::from_bytes(&tags).context("failed to parse mission tags")?;
    let traits =
        StrikeTeamTraits::from_bytes(&traits).context("failed to parse strike team traits")?;

    StrikeTeams::initialize(StrikeTeams {
        equipment,
        missions,
        specializations,
        tags,
        traits,
    });

    Ok(())
}

pub async fn initialize_items() -> anyhow::Result<()> {
    const INVENTORY_DEFINITIONS_RESOURCE: &str = "inventoryDefinitions.json";

    let definitions = Resources::get_resource_bytes(INVENTORY_DEFINITIONS_RESOURCE).await?;

    let definitions: Vec<ItemDefinition> =
        serde_json::from_slice(&definitions).context("Failed to load inventory definitions")?;

    Items::initialize(Items::from_definitions(definitions));
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::services::definitions::{initialize_items, initialize_strike_teams};

    #[tokio::test]
    async fn ensure_strike_teams_load() {
        initialize_strike_teams().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_items_load() {
        initialize_items().await.unwrap();
    }
}
