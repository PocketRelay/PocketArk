use anyhow::Context;
use pocket_ark_definitions::{
    badges::Badges,
    challenges::{ChallengeDefinition, Challenges},
    classes::{Class, Classes},
    items::{ItemDefinition, Items},
    level_tables::{LevelTable, LevelTables},
    strike_teams::{
        StrikeTeams,
        equipment::StrikeTeamEquipmentList,
        mission::{MissionDefinitions, tag::MissionTags},
        specialization::StrikeTeamSpecializations,
        traits::StrikeTeamTraits,
    },
};
use pocket_ark_i18n::I18n;

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
    tracing::debug!("Loaded {} item definition(s)", definitions.len());

    Items::initialize(Items::from_definitions(definitions));
    Ok(())
}

pub async fn initialize_classes() -> anyhow::Result<()> {
    const CLASSES_DEFINITIONS_RESOURCE: &str = "characterClasses.json";

    let definitions = Resources::get_resource_bytes(CLASSES_DEFINITIONS_RESOURCE).await?;
    let definitions: Vec<Class> =
        serde_json::from_slice(&definitions).context("Failed to load class definitions")?;
    tracing::debug!("Loaded {} class definition(s)", definitions.len());
    Classes::initialize(Classes::from_definitions(definitions));
    Ok(())
}

pub async fn initialize_level_tables() -> anyhow::Result<()> {
    const LEVEL_TABLES_RESOURCE: &str = "levelTables.json";

    let definitions = Resources::get_resource_bytes(LEVEL_TABLES_RESOURCE).await?;
    let definitions: Vec<LevelTable> =
        serde_json::from_slice(&definitions).context("Failed to parse level table definitions")?;

    tracing::debug!("Loaded {} level table definition(s)", definitions.len());
    LevelTables::initialize(LevelTables::from_definitions(definitions));
    Ok(())
}

pub async fn initialize_challenges() -> anyhow::Result<()> {
    const CHALLENGE_DEFINITIONS_RESOURCE: &str = "challengeDefinitions.json";

    let definitions = Resources::get_resource_bytes(CHALLENGE_DEFINITIONS_RESOURCE).await?;
    let definitions: Vec<ChallengeDefinition> =
        serde_json::from_slice(&definitions).context("Failed to parse challenge definitions")?;

    tracing::debug!("Loaded {} challenge definition(s)", definitions.len());
    Challenges::initialize(Challenges::from_definitions(definitions));
    Ok(())
}

pub async fn initialize_i18n() -> anyhow::Result<()> {
    const EN_LOCALE_RESOURCE: &str = "locales/en.csv";

    let bytes = Resources::get_resource_bytes(EN_LOCALE_RESOURCE).await?;
    let i18n = I18n::from_csv_bytes(&bytes)?;

    tracing::debug!("Loaded {} translation(s)", i18n.count());
    I18n::initialize(i18n);
    Ok(())
}

pub async fn initialize_badges() -> anyhow::Result<()> {
    const BADGES_RESOURCE: &str = "matchBadges.json";

    let definitions = Resources::get_resource_bytes(BADGES_RESOURCE).await?;
    let definitions: Vec<Badge> =
        serde_json::from_slice(&definitions).context("Failed to parse badge definitions")?;

    tracing::debug!("Loaded {} badge definition(s)", definitions.len());
    Badges::initialize(Badges::from_definitions(definitions));
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::services::definitions::{
        initialize_challenges, initialize_classes, initialize_i18n, initialize_items,
        initialize_level_tables, initialize_strike_teams,
    };

    #[tokio::test]
    async fn ensure_strike_teams_load() {
        initialize_strike_teams().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_items_load() {
        initialize_items().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_classes_load() {
        initialize_classes().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_level_tables_load() {
        initialize_level_tables().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_challenges_load() {
        initialize_challenges().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_i18n_load() {
        initialize_i18n().await.unwrap();
    }

    #[tokio::test]
    async fn ensure_badges_load() {
        initialize_badges().await.unwrap();
    }
}
