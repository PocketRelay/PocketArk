use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, DisplayFromStr};
use std::{
    fmt::{Display, Write},
    str::FromStr,
};
use thiserror::Error;

use crate::definitions::items::{
    ItemName,
    category::{BaseCategory, BaseCategoryError},
};

/// Link to an item, contains the item category and [ItemName]
#[derive(Debug)]
pub struct ItemLink(pub BaseCategory, pub ItemName);

impl Display for ItemLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)?;
        f.write_char(':')?;
        Display::fmt(&self.1, f)
    }
}

impl<'de> Deserialize<'de> for ItemLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}

impl Serialize for ItemLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Errors that can occur when parsing an [ItemLink]
#[derive(Debug, Error)]
pub enum ItemLinkError {
    /// Error parsing the category portion
    #[error(transparent)]
    Base(#[from] BaseCategoryError),
    /// Item name portion of the link is missing
    #[error("Item link missing item name")]
    MissingName,
    /// Error parsing the item name
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl FromStr for ItemLink {
    type Err = ItemLinkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, name) = s.split_once(':').ok_or(ItemLinkError::MissingName)?;
        let base: BaseCategory = base.parse()?;
        let name: ItemName = name.parse()?;

        Ok(Self(base, name))
    }
}
