use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize_f64_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let val = <f64 as Deserialize<'de>>::deserialize(deserializer)?;
    Ok(val as u32)
}

pub fn serialize_f64_u32<S>(val: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(*val as f64)
}
