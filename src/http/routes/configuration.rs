use crate::http::models::RawJson;

/// Overall configuration for multiplayer
static CONFIGURATION: &str = include_str!("../../resources/data/configuration.json");

/// GET /configuration
///
/// Obtains the configuration definition
pub async fn get_configuration() -> RawJson {
    RawJson(CONFIGURATION)
}
