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
pub struct Rule {
    // Game attribute represented by the rule
    attr: &'static str,
    rnd_attr: &'static str,
    mft_attr: &'static str,
}

impl Rule {
    const fn new(attr: &'static str, rnd_attr: &'static str, mft_attr: &'static str) -> Self {
        Self {
            attr,
            rnd_attr,
            mft_attr,
        }
    }
}

/// Known rules and the attribute they operate over
pub static RULES: &[Rule] = &[
    Rule::new("difficulty", "difficultyRND", "difficultyMFT"),
    Rule::new("enemytype", "enemytypeRND", "enemytypeMFT"),
    Rule::new("level", "levelRND", "levelMFT"),
];

/// Attribute determining the game privacy for public
/// match checking
const PRIVACY_ATTR: &str = "coopGameVisibility";

/// Defines a rule to be matched and the value to match
#[derive(Debug)]
pub struct MatchRule {
    /// Rule being matched for
    rule: &'static Rule,
    /// Value to match using
    value: String,
    /// Random value to use
    rand_value: Option<String>,
    /// Mode to perform the match with
    match_mode: Option<String>,
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
            let attr = pairs
                .iter()
                .find(|value| value.0.eq(rule.attr))
                .map(|value| value.1.clone());
            let rand_value = pairs
                .iter()
                .find(|value| value.0.eq(rule.rnd_attr))
                .map(|value| value.1.clone());
            let match_mode = pairs
                .iter()
                .find(|value| value.0.eq(rule.mft_attr))
                .map(|value| value.1.clone());

            if let Some(value) = attr {
                rules.push(MatchRule {
                    rule,
                    value,
                    rand_value,
                    match_mode,
                })
            }
        }

        Self { rules }
    }

    /// Checks if the rules provided in this rule set match the values in
    /// the attributes map.
    pub fn matches(&self, attributes: &AttrMap) -> bool {
        // Non public matches are unable to be matched
        if let Some(privacy) = attributes.get(PRIVACY_ATTR)
            && privacy != "1"
        {
            return false;
        }

        // Handle matching requested rules
        for rule in &self.rules {
            // Ensure the attribute is present and matching
            if !attributes
                .get(rule.rule.attr)
                .is_some_and(|value| value.eq(&rule.value))
            {
                return false;
            }
        }

        true
    }
}
