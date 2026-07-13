use crate::definitions::{
    items::{
        ItemDefinition, ItemName,
        category::{BaseCategory, Category},
        rarity::ItemRarity,
    },
    packs::parser::{FilterParseError, parse_filter},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{DeserializeAs, DisplayFromStr};
use std::{
    fmt::{Display, Write},
    str::FromStr,
};

/// Type used for the weight of a filter result
pub type Weight = u32;

/// Item filtering
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Filter that never matches anything (Fallback)
    Never,

    /// Specific item referenced by [ItemName]
    Named(ItemName),
    /// Require the item to be a specific rarity
    Rarity(ItemRarity),
    /// Item from a selection of a category
    Category(Category),
    /// Filter based on a specific item attribute
    Attribute(String, Value),

    /// Filter matching many filters. Only one of the filters needs to
    /// pass, will compare all the filters and the weight will become
    /// the sum of all matching filters
    Many(Vec<Filter>),
    /// Filter matching only when both filters match
    And(Box<Filter>, Box<Filter>),
    /// Filter matching when either filter matches
    Or(Box<Filter>, Box<Filter>),
    /// Filter requiring the other filter does not match
    Not(Box<Filter>),

    /// Filter with an additional weighted randomness amount
    Weighted(Box<Filter>, Weight),
}

impl FromStr for Filter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_filter(s)
    }
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DisplayFromStr::deserialize_as(deserializer)
    }
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Filter {
    fn fmt_attribute_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Named(_) => f.write_str("Name"),
            Filter::Rarity(_) => f.write_str("Rarity"),
            Filter::Category(_) => f.write_str("Category"),
            Filter::Attribute(key, _) => write!(f, "Attribute={key},"),
            _ => Ok(()),
        }
    }

    fn fmt_attribute_value(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Named(uuid) => write!(f, "(Name={uuid})"),
            Filter::Rarity(item_rarity) => write!(f, "{item_rarity}"),
            Filter::Category(category) => write!(f, "{category}"),
            Filter::Attribute(_, value) => match value {
                Value::String(value) => write!(f, "{value}"),
                _ => write!(f, "{value})"),
            },
            _ => Ok(()),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Never => write!(f, "Never"),
            Filter::Named(uuid) => write!(f, "Name={uuid}"),
            Filter::Rarity(item_rarity) => write!(f, "Rarity={item_rarity}"),
            Filter::Category(category) => write!(f, "Category={category}"),
            Filter::Attribute(key, value) => match value {
                Value::String(value) => write!(f, "Attribute={key},{value}"),
                _ => write!(f, "Attribute={key},{value}"),
            },
            Filter::Many(filters) => {
                let last_index = filters.len() - 1;

                for (index, filter) in filters.iter().enumerate() {
                    Display::fmt(filter, f)?;

                    if index != last_index {
                        f.write_char(',')?;
                    }
                }

                Ok(())
            }
            Filter::And(left, right) => write!(f, "({left} && {right})"),
            Filter::Or(left, right) => {
                if let Some(flattened) = FlattenedListFilter::try_flatten(left, right) {
                    write!(f, "{flattened}")
                } else {
                    write!(f, "({left} || {right})")
                }
            }
            Filter::Not(filter) => write!(f, "(!{filter})"),
            Filter::Weighted(filter, weight) => write!(f, "({filter}^{weight})"),
        }
    }
}

/// Attributes set flattened into a list with a trailing condition
struct FlattenedListFilter<'a> {
    /// Root attribute filter (leftmost of the OR condition)
    attribute: &'a Filter,
    /// Values aligned to the same attribute
    values: Vec<&'a Filter>,
    /// Remaining right hand filter if a portion of the set does not align with the
    /// leftmost condition
    remaining: Option<Filter>,
}

impl<'a> FlattenedListFilter<'a> {
    pub fn try_flatten(left: &'a Filter, right: &'a Filter) -> Option<FlattenedListFilter<'a>> {
        let mut values = Vec::new();
        let mut current_left = left;

        values.push(right);

        while let Filter::Or(next_left, next_right) = current_left {
            values.push(next_right);
            current_left = next_left;
        }

        let anchor_attribute = current_left;
        if !anchor_attribute.is_attribute_filter() {
            return None;
        }

        values.reverse();

        let mut split_index = 0;
        while split_index < values.len()
            && anchor_attribute.is_matching_attribute(values[split_index])
        {
            split_index += 1;
        }

        if split_index == 0 {
            return None;
        }

        let remaining_values = values.split_off(split_index);
        let remaining = if remaining_values.is_empty() {
            None
        } else {
            // Rebuild a left-nested OR chain from the leftovers
            let mut iter = remaining_values.into_iter();
            let mut combined = iter.next().unwrap().clone();
            for next_filter in iter {
                combined = Filter::or(combined.clone(), next_filter.clone());
            }
            Some(combined)
        };

        Some(FlattenedListFilter {
            attribute: anchor_attribute,
            values,
            remaining,
        })
    }

    fn fmt_attributes_list(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.attribute.fmt_attribute_name(f)?;
        f.write_str("=[")?;
        self.attribute.fmt_attribute_value(f)?;

        for value in &self.values {
            f.write_char(',')?;
            value.fmt_attribute_value(f)?;
        }

        f.write_str("]")?;

        Ok(())
    }
}

impl<'a> Display for FlattenedListFilter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(remaining) = self.remaining.as_ref() {
            f.write_char('(')?;
            self.fmt_attributes_list(f)?;
            f.write_str(" || ")?;
            remaining.fmt(f)?;
            f.write_char(')')?;
        }

        self.fmt_attributes_list(f)
    }
}

#[allow(unused)]
impl Filter {
    pub fn is_attribute_filter(&self) -> bool {
        matches!(
            self,
            Filter::Named(_) | Filter::Rarity(_) | Filter::Category(_) | Filter::Attribute(_, _)
        )
    }

    pub fn is_matching_attribute(&self, other: &Filter) -> bool {
        match (self, other) {
            // Both branches match a name
            (Filter::Named(_), Filter::Named(_)) |

            // Both branch match a rarity
            (Filter::Rarity(_), Filter::Rarity(_)) |

            // Both branch match a category
            (Filter::Category(_), Filter::Category(_)) => true,

            // Both branches match the same attribute key
            (Filter::Attribute(key, _), Filter::Attribute(other_key, _)) => key.eq(other_key),

            _ => false,
        }
    }

    /// Creates a new filter matching all of the provided filters
    pub fn all<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Filter>,
    {
        filters
            .into_iter()
            .reduce(|accum, value| accum.and(value))
            .unwrap_or(Filter::Never)
    }

    /// Creates a new filter matching any of the provided filters
    pub fn any<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Filter>,
    {
        filters
            .into_iter()
            .reduce(|accum, value| accum.or(value))
            .unwrap_or(Filter::Never)
    }

    /// Creates a filter that matches all the provided `rarities`
    pub fn rarities<I>(rarities: I) -> Self
    where
        I: IntoIterator<Item = ItemRarity>,
    {
        Self::any(rarities.into_iter().map(Self::Rarity))
    }

    /// Filter that accepts any rarity
    pub fn any_rarity() -> Self {
        Self::rarities([
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::UltraRare,
        ])
    }

    /// Creates a filter that matches all the provided `categories`
    pub fn categories<I>(categories: I) -> Self
    where
        I: IntoIterator<Item = Category>,
    {
        Self::any(categories.into_iter().map(Self::Category))
    }

    #[inline]
    pub const fn base_category(category: BaseCategory) -> Self {
        Self::Category(Category::Base(category))
    }

    /// Creates a filter that matches all the provided `base_categories`
    pub fn base_categories<I>(categories: I) -> Self
    where
        I: IntoIterator<Item = BaseCategory>,
    {
        Self::any(categories.into_iter().map(Self::base_category))
    }

    /// Creates an attribute filter from the provided key and value
    pub fn attribute<K, V>(key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        Self::Attribute(key.into(), value.into())
    }

    /// Creates an attributes filter from an iterator of key
    /// value pairs requires all the attribute match
    pub fn attributes<I, K, V>(attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        Self::all(
            attributes
                .into_iter()
                .map(|(key, value)| Self::attribute(key, value)),
        )
    }

    /// Combines the current filter with another filter using
    /// AND logic
    pub fn and(self, other: Self) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    /// Combines the current filter with another filter using
    /// OR logic
    pub fn or(self, other: Self) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    /// Inverts the current filter
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Applies a weight to the filter
    pub fn weight(self, weight: u32) -> Self {
        Self::Weighted(Box::new(self), weight)
    }

    /// Creates a new [Filter::Many] filter from an iterator
    /// of filters
    pub fn many<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Many(iter.into_iter().collect())
    }

    /// Combines the two filters, used to merge additional weights
    #[inline]
    pub fn merge(self, filter: Filter) -> Self {
        Self::many([self, filter])
    }

    /// Combines the many filters, used to merge additional weights
    #[inline]
    pub fn merge_many<I>(self, filters: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::many(Some(self).into_iter().chain(filters))
    }

    /// Applies the filter against the provided `item` definition
    /// returns [None] if the value did not match otherwise returns
    /// [Some] with the calculated [FilterWeight]
    pub fn apply_filter(&self, item: &ItemDefinition) -> Option<Weight> {
        match self {
            Filter::Named(name) => {
                if name != &item.name {
                    return None;
                }

                Some(0)
            }
            Filter::Rarity(rarity) => {
                let item_rarity = item.rarity.as_ref()?;
                if rarity != item_rarity {
                    return None;
                }

                Some(rarity.weight())
            }
            Filter::Category(category) => {
                let item_category = &item.category;

                if item_category.is_within(category) {
                    return None;
                }

                Some(0)
            }
            Filter::Attribute(key, value) => {
                let matches = item
                    .custom_attributes
                    .get(key)
                    .is_some_and(|attr| attr.eq(value));

                if matches { Some(0) } else { None }
            }
            Filter::Many(filters) => {
                let mut weight_sum = 0;
                let mut matches = false;

                for filter in filters {
                    if let Some(weight) = filter.apply_filter(item) {
                        weight_sum += weight;
                        matches = true;
                    }
                }

                if matches { Some(weight_sum) } else { None }
            }
            Filter::And(left, right) => {
                let left = left.apply_filter(item)?;
                let right = right.apply_filter(item)?;
                Some(left + right)
            }
            Filter::Or(left, right) => {
                if let Some(left) = left.apply_filter(item) {
                    Some(left)
                } else {
                    right.apply_filter(item)
                }
            }
            Filter::Not(filter) => {
                if filter.apply_filter(item).is_some() {
                    None
                } else {
                    Some(0)
                }
            }
            Filter::Weighted(filter, weight) => filter
                .apply_filter(item)
                // Add the additional weight
                .map(|filter_weight| filter_weight + *weight),
            Filter::Never => None,
        }
    }
}
