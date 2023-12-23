use sea_orm::prelude::DateTimeUtc;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_with::skip_serializing_none;

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
