use std::collections::HashMap;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

pub mod characters;
pub mod class_data;
pub mod currency;
pub mod inventory_items;
pub mod seen_articles;
pub mod shared_data;
pub mod users;

pub type Character = characters::Model;
pub type CharacterEntity = characters::Entity;

pub type Currency = currency::Model;
pub type CurrencyEnitty = currency::Entity;

pub type SharedData = shared_data::Model;
pub type SharedDataEntity = shared_data::Entity;

pub type InventoryItem = inventory_items::Model;
pub type InventoryItemEntity = inventory_items::Entity;

pub type User = users::Model;
pub type UserEntity = users::Entity;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct ValueMap(pub HashMap<String, serde_json::Value>);
