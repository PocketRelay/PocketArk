use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::Map;

pub mod challenge_progress;
pub mod characters;
pub mod class_data;
pub mod currency;
pub mod inventory_items;
pub mod seen_articles;
pub mod shared_data;
pub mod users;

pub type Character = characters::Model;
pub type CharacterEntity = characters::Entity;

pub type ChallengeProgress = challenge_progress::Model;

pub type Currency = currency::Model;
pub type CurrencyEnitty = currency::Entity;

pub type SharedData = shared_data::Model;
pub type SharedDataEntity = shared_data::Entity;

pub type InventoryItem = inventory_items::Model;
pub type InventoryItemEntity = inventory_items::Entity;

pub type User = users::Model;
pub type UserEntity = users::Entity;

pub type ClassData = class_data::Model;
pub type ClassDataEntity = class_data::Entity;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct ValueMap(pub Map<String, serde_json::Value>);
