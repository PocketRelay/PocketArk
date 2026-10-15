use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents a duration of time that something will be available for.
/// Can be open ended by only specifying a start/end
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDuration {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}
