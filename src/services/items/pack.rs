//! Pack generation scripts, used to generate the lists of weighted
//! items for randomly generated packs.
//!
//! The randomness used for these packs are only guesses and may not
//! be accurate to the actual game loot tables.

mod filter {
    //! Item filtering based on its definition

    use std::collections::HashMap;

    use serde_json::Value;

    use crate::services::items::v2::{Category, ItemDefinition, ItemName, ItemRarity};

    /// Type used for the weight of a filter result
    type FilterWeight = u32;

    /// Item filtering
    #[derive(Debug, Clone)]
    pub enum ItemFilter {
        /// Specific item referenced by [ItemName]
        Named(ItemName),
        /// Require the item to be a specific rarity
        Rarity(ItemRarity),
        /// Item from a selection of a category
        Category(Category),
        /// Filter based on matching item attributes
        Attributes(HashMap<String, Value>),

        /// Filter matching any of the provided filters
        Any(Vec<ItemFilter>),
        /// Filter matching only when both filters match
        And(Box<ItemFilter>, Box<ItemFilter>),
        /// Filter matching when either filter matches
        Or(Box<ItemFilter>, Box<ItemFilter>),
        /// Filter requiring the other filter does not match
        Not(Box<ItemFilter>),

        /// Filter with an additional weighted randomness amount
        Weighted(Box<ItemFilter>, FilterWeight),
    }

    impl ItemFilter {
        /// Creates a filter that matches all the provided `rarities`
        pub fn rarities(rarities: &[ItemRarity]) -> Self {
            Self::Any(rarities.iter().map(|value| Self::Rarity(*value)).collect())
        }

        /// Creates a filter that matches all the provided `categories`
        pub fn categories(categories: &[Category]) -> Self {
            Self::Any(
                categories
                    .iter()
                    .map(|value| Self::Category(value.clone()))
                    .collect(),
            )
        }

        /// Creates an attributes filter from an iterator of key
        /// value pairs
        pub fn attributes<I, K, V>(attributes: I) -> Self
        where
            I: IntoIterator<Item = (K, V)>,
            K: Into<String>,
            V: Into<Value>,
        {
            Self::Attributes(
                attributes
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
            )
        }

        /// Applies the filter against the provided `item` definition
        /// returns [None] if the value did not match otherwise returns
        /// [Some] with the calculated [FilterWeight]
        pub fn apply_filter(&self, item: &ItemDefinition) -> Option<FilterWeight> {
            match self {
                ItemFilter::Named(name) => {
                    if name != &item.name {
                        return None;
                    }

                    Some(0)
                }
                ItemFilter::Rarity(rarity) => {
                    let item_rarity = item.rarity.as_ref()?;
                    if rarity != item_rarity {
                        return None;
                    }

                    Some(0)
                }
                ItemFilter::Category(category) => {
                    let item_category = &item.category;

                    if item_category.is_within(category) {
                        return None;
                    }

                    Some(0)
                }
                ItemFilter::Attributes(attributes) => {}
                ItemFilter::Any(_) => todo!(),
                ItemFilter::And(_, _) => todo!(),
                ItemFilter::Or(_, _) => todo!(),
                ItemFilter::Not(_) => todo!(),
                ItemFilter::Weighted(_, _) => todo!(),
            }
        }
    }
}
