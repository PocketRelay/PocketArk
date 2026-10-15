use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the name for an activity, contains built in
/// server activity types along with the [Uuid] variant for
/// runtime defined activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityName {
    /// Item was consumed
    ///
    /// Known attributes:
    /// - category (string)
    /// - definitionName (string uuid)
    /// - count (number)
    #[serde(rename = "_itemConsumed")]
    ItemConsumed,
    /// Badge was earned on game completion
    ///
    /// Known attributes:
    /// - badgeName (string)
    /// - count (number)
    #[serde(rename = "_badgeEarned")]
    BadgeEarned,
    /// Article was purchased from the store
    ///
    /// Known attributes:
    /// - currencyName (string)
    /// - articleName (string uuid)
    /// - count (number)
    #[serde(rename = "_articlePurchased")]
    ArticlePurchased,
    /// Mission was finished
    ///
    /// Known attributes:
    /// - percentComplete (number)
    /// - missionTypeName (string uuid)
    /// - count (number)
    #[serde(rename = "_missionFinished")]
    MissionFinished,
    /// Mission was finished by a strike team
    ///
    /// Known attributes:
    /// - success (string boolean)
    /// - count (number)
    #[serde(rename = "_strikeTeamMissionFinished")]
    StrikeTeamMissionFinished,
    /// Equipment was updated
    ///
    /// Known attributes:
    /// - slot (string)
    /// - count (number)
    /// - stackSize (number)
    #[serde(rename = "_equipmentUpdated")]
    EquipmentUpdated,
    /// Equipment attachments were updated
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_equipmentAttachmentUpdated")]
    EquipmentAttachmentUpdated,
    /// Skills were purchased
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_skillPurchased")]
    SkillPurchased,
    /// Character was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - characterClass (string uuid)
    /// - count (number)
    #[serde(rename = "_characterLevelUp")]
    CharacterLevelUp,
    /// Prestige was leveled up
    ///
    /// Known attributes:
    /// - newLevel (number)
    /// - count (number)
    #[serde(rename = "_prestigeLevelUp")]
    PrestigeLevelUp,
    /// Pathfinder rating has changed
    ///
    /// Known attributes
    /// - pathfinderRatingDelta (number)
    #[serde(rename = "_pathfinderRatingUpdated")]
    PathfinderRatingUpdated,
    /// Strike team was recruited
    ///
    /// Known attributes:
    /// - count (number)
    #[serde(rename = "_strikeTeamRecruited")]
    StrikeTeamRecruited,
    /// Activity represented by a [Uuid] these events can be
    /// published by clients
    #[serde(untagged)]
    Named(Uuid),
}

/// Type alias for a string representing an attribute name
pub type AttributeName = String;

/// Represents a published activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// The name of the activity event
    pub name: ActivityName,
    /// Data attributes associated with this activity event
    pub attributes: HashMap<AttributeName, ActivityAttribute>,
}

impl ActivityEvent {
    /// Creates a new activity event
    pub fn new(name: ActivityName) -> Self {
        Self {
            name,
            attributes: Default::default(),
        }
    }

    /// Adds an attribute to an activity event
    pub fn with_attribute<V>(mut self, key: &str, value: V) -> Self
    where
        V: Into<ActivityAttribute>,
    {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    #[allow(unused)]
    pub fn attribute_string(&self, key: &str) -> Result<&String, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::String(value) => Ok(value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    /// Obtains an attribute by attempting to parse it
    /// from a [ActivityAttribute::String] value
    pub fn attribute_parsed<V>(&self, key: &str) -> Result<V, AttributeError>
    where
        V: FromStr,
        <V as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        let value = match attribute {
            ActivityAttribute::String(value) => value,
            _ => return Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        };

        value
            .parse()
            // Handle parsing error
            .map_err(|err| {
                AttributeError::new(key, AttributeErrorCause::ParseFailed(Box::new(err)))
            })
    }

    pub fn attribute_uuid(&self, key: &str) -> Result<Uuid, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::Uuid(value) => Ok(*value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    pub fn attribute_u32(&self, key: &str) -> Result<u32, AttributeError> {
        let attribute = self
            .attributes
            .get(key)
            .ok_or(AttributeError::new(key, AttributeErrorCause::Missing))?;

        match attribute {
            ActivityAttribute::Integer(value) => Ok(*value),
            _ => Err(AttributeError::new(key, AttributeErrorCause::IncorrectType)),
        }
    }

    /// Obtains the score from the mission activity if it
    /// is present within the attributes
    #[inline]
    pub fn get_score(&self) -> Option<u32> {
        self.attribute_u32("score").ok()
    }

    /// Checks if this activity `attributes` match the provided filter
    pub fn matches_filter(&self, filter: &HashMap<AttributeName, ActivityFilter>) -> bool {
        filter
            .iter()
            // Ensure all attributes match
            .all(|(key, filter)| {
                self.attributes
                    .get(key)
                    // Ensure the value exists and matches
                    .is_some_and(|value| filter.matches(value))
            })
    }
}

/// Represents an attribute within an [ActivityEvent]. These
/// can be numbers or strings
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityAttribute {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Integer(u32),
    /// UUID value
    Uuid(Uuid),
    /// String value
    String(String),
}

impl From<u32> for ActivityAttribute {
    fn from(value: u32) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for ActivityAttribute {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ActivityAttribute {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Uuid> for ActivityAttribute {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<bool> for ActivityAttribute {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl PartialEq for ActivityAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Simple equality
            (Self::Bool(left), Self::Bool(right)) => left.eq(right),
            (Self::Integer(left), Self::Integer(right)) => left.eq(right),
            (Self::String(left), Self::String(right)) => left.eq(right),
            (Self::Uuid(left), Self::Uuid(right)) => left.eq(right),

            // Additional equality for UUID strings (Can be removed once types are strict)
            (Self::Uuid(left), Self::String(right)) => left.to_string().eq(right),
            (Self::String(left), Self::Uuid(right)) => left.eq(&right.to_string()),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct AttributeError {
    /// Name of the attribute
    name: AttributeName,
    /// Cause of the error
    cause: AttributeErrorCause,
}

impl AttributeError {
    fn new(name: &str, cause: AttributeErrorCause) -> Self {
        Self {
            name: name.to_string(),
            cause,
        }
    }
}

#[derive(Debug)]
pub enum AttributeErrorCause {
    /// Attribute was not found
    Missing,
    /// Attribute was an unexpected type
    IncorrectType,
    /// Failed to parse the value
    ParseFailed(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl std::error::Error for AttributeError {}

impl Display for AttributeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error with attribute '{}': {}",
            self.name, self.cause
        ))
    }
}

impl Display for AttributeErrorCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeErrorCause::Missing => f.write_str("Attribute is missing"),
            AttributeErrorCause::IncorrectType => f.write_str("Unexpected attribute type"),
            AttributeErrorCause::ParseFailed(err) => {
                f.write_str("Failed to parse: ")?;
                Display::fmt(err, f)
            }
        }
    }
}

/// Describes an activity that can be used to track progress
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDescriptor {
    /// Name of the [ActivityEvent] this descriptor is for
    /// (Can be a [Uuid] or just text such as: "_itemConsumed")
    pub activity_name: ActivityName,
    /// Filtering based on the [ActivityEvent::attributes] for
    /// whether the activity is applicable
    pub filter: HashMap<String, ActivityFilter>,
    /// The key into [ActivityEvent::attributes] that should be
    /// used for tracking activity progress
    #[serde(rename = "incrementProgressBy")]
    pub progress_key: String,
}

/// Enum for different ways an activity can be filtered against
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ActivityFilter {
    /// Direct value comparison
    Value(ActivityAttribute),
    /// Not equal comparison
    NotEqual {
        /// The value to compare not equal against
        #[serde(rename = "$ne")]
        ne: ActivityAttribute,
    },
}

impl ActivityFilter {
    /// Checks whether the provided [ActivityAttribute] matches this filter
    pub fn matches(&self, other: &ActivityAttribute) -> bool {
        match self {
            Self::Value(value) => value.eq(other),
            Self::NotEqual { ne } => ne.ne(other),
        }
    }
}

impl ActivityDescriptor {
    /// Checks if the provided `activity` matches this descriptor
    pub fn matches(&self, activity: &ActivityEvent) -> bool {
        self.activity_name.eq(&activity.name) && activity.matches_filter(&self.filter)
    }
}
