//! Field used to determine how match happens is: {field}MFT
//! - When not specified, value must be an exact match
//! - Can also be "matchAny" to match any value
//! - When matching exact value check against {field}
//!
//! {field}RND possibly a random choice by the client? For the server to use
//! if there are no games when creating the game
//!
//! GameSize is matchAny when performing a quick match
//!
//! coopGameVisibility (0 = Private, 1 = Public)
//!
//! Searching for mission specific ones set mission type to a uuid
//! mission slot to the mission specific slot, sets modifiers count to an amount
//! and stores a number like 00000004 in the modifiers field sets the name to a uuid
//!
//!

use super::AttrMap;

#[derive(Debug)]
pub enum Rule {
    Match {
        /// Attribute containing the value
        attr: &'static str,
        /// Attribute containing the random default value
        rand_attr: &'static str,
        /// Attribute containing the match mode
        mode_attr: &'static str,
    },

    /// Exact match attribute
    ExactField {
        /// Attribute containing the value
        attr: &'static str,
    },

    /// Special rule for the "GameSize" rule
    GameSize,
}

/// Known rules and the attribute they operate over
pub static RULES: &[Rule] = &[
    Rule::Match {
        attr: "difficulty",
        rand_attr: "difficultyRND",
        mode_attr: "difficultyMFT",
    },
    Rule::Match {
        attr: "enemytype",
        rand_attr: "enemytypeRND",
        mode_attr: "enemytypeMFT",
    },
    Rule::Match {
        attr: "level",
        rand_attr: "levelRND",
        mode_attr: "levelMFT",
    },
    // Game visibility
    Rule::ExactField {
        attr: "coopGameVisibility",
    },
    // Mission
    Rule::ExactField {
        attr: "missionSlot",
    },
    Rule::ExactField {
        attr: "modifierCount",
    },
    Rule::ExactField { attr: "modifiers" },
    Rule::GameSize,
];

/// Attribute determining the game privacy for public
/// match checking
const PRIVACY_ATTR: &str = "coopGameVisibility";

/// Defines a rule to be matched and the value to match
#[derive(Debug)]
pub struct MatchRule {
    /// Rule being matched for
    rule: &'static Rule,

    value: MatchRuleValue,
}

#[derive(Debug)]
pub enum MatchRuleValue {
    MatchRule {
        /// Value to match using
        value: String,
        /// Random value to use
        #[allow(unused)]
        rand_value: Option<String>,
        /// Mode to perform the match with
        match_mode: Option<String>,
    },
    Value(String),
}

/// Set of rules to match
#[derive(Debug)]
pub struct RuleSet {
    /// The rules to match
    rules: Vec<MatchRule>,
}

impl RuleSet {
    /// Creates a new set of rule matches from the provided rule value pairs
    pub fn new(pairs: Vec<(String, String)>) -> Self {
        let mut rules = Vec::new();

        for rule in RULES {
            match rule {
                Rule::Match {
                    attr,
                    rand_attr,
                    mode_attr,
                } => {
                    let attr = pairs
                        .iter()
                        .find(|value| value.0.eq(attr))
                        .map(|value| value.1.clone());
                    let rand_value = pairs
                        .iter()
                        .find(|value| value.0.eq(rand_attr))
                        .map(|value| value.1.clone());
                    let match_mode = pairs
                        .iter()
                        .find(|value| value.0.eq(mode_attr))
                        .map(|value| value.1.clone());

                    if let Some(value) = attr {
                        rules.push(MatchRule {
                            rule,
                            value: MatchRuleValue::MatchRule {
                                value,
                                rand_value,
                                match_mode,
                            },
                        })
                    }
                }

                Rule::ExactField { attr } => {
                    let attr = pairs
                        .iter()
                        .find(|value| value.0.eq(attr))
                        .map(|value| value.1.clone());

                    if let Some(attr) = attr {
                        rules.push(MatchRule {
                            rule,
                            value: MatchRuleValue::Value(attr),
                        });
                    }
                }

                Rule::GameSize => {
                    let attr = pairs
                        .iter()
                        .find(|value| value.0.eq("GameSize"))
                        .map(|value| value.1.clone());

                    if let Some(attr) = attr {
                        rules.push(MatchRule {
                            rule,
                            value: MatchRuleValue::Value(attr),
                        });
                    }
                }
            }
        }

        Self { rules }
    }

    /// Checks if the rules provided in this rule set match the values in
    /// the attributes map.
    pub fn matches(&self, attributes: &AttrMap, game_size: usize) -> bool {
        // Non public matches are unable to be matched
        if let Some(privacy) = attributes.get(PRIVACY_ATTR)
            && privacy != "1"
        {
            return false;
        }

        // Handle matching requested rules
        for rule in &self.rules {
            match (rule.rule, &rule.value) {
                (
                    Rule::Match { attr, .. },
                    MatchRuleValue::MatchRule {
                        value, match_mode, ..
                    },
                ) => {
                    // We don't care what the value is
                    if match_mode.as_ref().is_some_and(|value| value == "matchAny") {
                        continue;
                    }

                    // Ensure the attribute is present and matching
                    if !attributes
                        .get(*attr)
                        .is_some_and(|attr_value| attr_value.eq(value))
                    {
                        return false;
                    }
                }

                (Rule::ExactField { attr }, MatchRuleValue::Value(value)) => {
                    // Ensure the attribute is present and matching
                    if !attributes
                        .get(*attr)
                        .is_some_and(|attr_value| attr_value.eq(value))
                    {
                        return false;
                    }
                }

                (Rule::GameSize, MatchRuleValue::Value(value)) => {
                    if value != "matchAny" && value != &game_size.to_string() {
                        return false;
                    }
                }

                _ => {
                    // unexpected case
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use crate::services::game::AttrMap;

    use super::RuleSet;

    /// Public match should succeed if the attributes meet the specified criteria
    #[test]
    fn test_public_match() {
        let attributes = [
            ("coopGameVisibility", "1"),
            ("difficulty", "1"),
            ("difficultyRND", ""),
            ("difficultyUI", "1"),
            ("enemytype", "0"),
            ("enemytypeRND", "2"),
            ("enemytypeUI", "0"),
            ("isInLobby", "true"),
            ("level", "7"),
            ("levelRND", ""),
            ("levelUI", "7"),
            ("lockState", "0"),
            ("missionSlot", "0"),
            ("missiontype", "Custom"),
            ("mode", "contact_multiplayer"),
            ("modifierCount", "0"),
            ("modifiers", ""),
            ("name", "Custom"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<AttrMap>();

        let rules = [
            ("GameSize", "matchAny"),
            ("coopGameVisibility", "1"),
            ("difficulty", "1"),
            ("difficultyRND", ""),
            ("enemytype", "0"),
            ("enemytypeMFT", "matchAny"),
            ("enemytypeRND", "2"),
            ("level", "0"),
            ("levelMFT", "matchAny"),
            ("levelRND", "13"),
            ("missionSlot", "0"),
            ("modifierCount", "0"),
            ("modifiers", ""),
            ("name", ""),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<(String, String)>>();

        let rule_set = RuleSet::new(rules);

        let matches = rule_set.matches(&attributes, 4);

        assert!(matches, "Rule set didn't match the provided attributes");
    }

    /// Private match should never match
    #[test]
    fn test_private_match() {
        let attributes = [
            ("coopGameVisibility", "0"),
            ("difficulty", "1"),
            ("difficultyRND", ""),
            ("difficultyUI", "1"),
            ("enemytype", "0"),
            ("enemytypeRND", "2"),
            ("enemytypeUI", "0"),
            ("isInLobby", "true"),
            ("level", "7"),
            ("levelRND", ""),
            ("levelUI", "7"),
            ("lockState", "0"),
            ("missionSlot", "0"),
            ("missiontype", "Custom"),
            ("mode", "contact_multiplayer"),
            ("modifierCount", "0"),
            ("modifiers", ""),
            ("name", "Custom"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<AttrMap>();

        let rules = [
            ("GameSize", "matchAny"),
            ("coopGameVisibility", "1"),
            ("difficulty", "1"),
            ("difficultyRND", ""),
            ("enemytype", "0"),
            ("enemytypeMFT", "matchAny"),
            ("enemytypeRND", "2"),
            ("level", "0"),
            ("levelMFT", "matchAny"),
            ("levelRND", "13"),
            ("missionSlot", "0"),
            ("modifierCount", "0"),
            ("modifiers", ""),
            ("name", ""),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<(String, String)>>();

        let rule_set = RuleSet::new(rules);

        let matches = rule_set.matches(&attributes, 4);

        assert!(!matches, "Rule set shouldn't match the provided attributes");
    }
}
