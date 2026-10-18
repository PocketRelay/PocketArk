use anyhow::Context;

use crate::{
    http::models::{DynHttpError, RawJsonBytes},
    resources::Resources,
};

const CONFIGURATION_RESOURCE: &str = "configuration.json";

/// GET /configuration
///
/// Obtains the configuration definition
pub async fn get_configuration() -> Result<RawJsonBytes, DynHttpError> {
    let data = Resources::get_resource_bytes(CONFIGURATION_RESOURCE)
        .await
        .context("failed to load configuration")?;
    Ok(RawJsonBytes(data))
}
