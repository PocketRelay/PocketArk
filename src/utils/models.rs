use crate::state::App;
use sea_orm::prelude::DateTimeUtc;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_with::skip_serializing_none;

/// Localized naming variables
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleNameWithDesc {
    // translation codes
    pub i18n_name: String,
    pub i18n_description: Option<String>,

    // translated
    pub loc_name: Option<String>,
    pub loc_description: Option<String>,
}

impl LocaleNameWithDesc {
    /// Returns the localized name if present otherwise
    /// returns the i18n translation code
    pub fn name(&self) -> &str {
        match self.loc_name.as_ref() {
            Some(value) => value,
            None => &self.i18n_name,
        }
    }

    /// Returns the localized description if present otherwise
    /// returns the i18n translation code
    pub fn description(&self) -> Option<&String> {
        self.loc_description
            .as_ref()
            .or(self.i18n_description.as_ref())
    }
}

/// Localized naming variables
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleName {
    // translation codes
    pub i18n_name: String,
    // translated
    pub loc_name: Option<String>,
}

impl LocaleName {
    /// Returns the localized name if present otherwise
    /// returns the i18n translation code
    pub fn name(&self) -> &str {
        match self.loc_name.as_ref() {
            Some(value) => value,
            None => &self.i18n_name,
        }
    }

    pub fn resolve(i18n: u32) -> Self {
        let services = App::services();
        let loc_name = services.i18n.get(i18n).map(|value| value.to_string());
        Self {
            i18n_name: i18n.to_string(),
            loc_name,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sku;

impl Serialize for Sku {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut this = serializer.serialize_struct("Sku", 2)?;
        this.serialize_field("title", "mec.game")?;
        this.serialize_field("platform", "origin")?;
        this.end()
    }
}

impl<'de> Deserialize<'de> for Sku {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NopVisitor;

        impl<'de> Visitor<'de> for NopVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sku")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                loop {
                    let entry = map.next_entry::<&str, &str>()?;
                    if entry.is_none() {
                        break;
                    }
                }
                Ok(())
            }
        }

        deserializer.deserialize_struct("Sku", &["title", "platform"], NopVisitor)?;

        Ok(Self)
    }
}

/// Represents a duration of time that something will be available for.
/// Can be open ended by only specifying a start/end
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDuration {
    pub start: Option<DateTimeUtc>,
    pub end: Option<DateTimeUtc>,
}
