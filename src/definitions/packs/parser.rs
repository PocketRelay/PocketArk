//! Pack filters have a specific parser format that simplifies the
//! structure that needs to be stored. This module specifies the
//! parser for that format

use crate::definitions::{
    items::{
        ItemName,
        category::{Category, CategoryError},
        rarity::ItemRarity,
    },
    packs::Filter,
};
use std::{
    iter::Peekable,
    mem::swap,
    num::ParseIntError,
    str::{Chars, FromStr},
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    /// Some key / value
    Value(String),

    /// Operator
    Operator(OperatorToken),

    /// Opening parenthesis
    ParenOpen,
    /// Closing parenthesis
    ParenClose,

    /// Weight specifying token has begun
    Weight,

    /// List open
    ListOpen,
    /// List close
    ListClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorToken {
    /// AND operator
    And,
    /// OR operator
    Or,
    /// Not operator
    Not,
    /// Comma separator
    Comma,
}

impl Token {
    fn as_value(&self) -> Option<&str> {
        match self {
            Token::Value(value) => Some(value),
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    current_value: String,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    fn tokenize_value(value: &str) -> Vec<Token> {
        let tokenizer = Tokenizer::from_str(value);
        tokenizer.tokenize()
    }

    fn from_str(value: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            chars: value.chars().peekable(),
            current_value: String::new(),
            tokens: Vec::new(),
        }
    }

    fn end_value(&mut self) {
        if !self.current_value.is_empty() {
            let mut value = String::new();
            swap(&mut value, &mut self.current_value);
            self.tokens.push(Token::Value(value));
        }
    }

    fn tokenize(mut self) -> Vec<Token> {
        while let Some(ch) = self.chars.next() {
            if ch.is_whitespace() {
                self.end_value();
                continue;
            }

            match ch {
                '=' => {
                    self.end_value();
                }
                ',' => {
                    self.end_value();
                    self.tokens.push(Token::Operator(OperatorToken::Comma));
                }
                '|' if self.chars.peek() == Some(&'|') => {
                    self.end_value();

                    self.tokens.push(Token::Operator(OperatorToken::Or));
                    self.chars.next();
                }
                '&' if self.chars.peek() == Some(&'&') => {
                    self.end_value();

                    self.tokens.push(Token::Operator(OperatorToken::And));
                    self.chars.next();
                }
                '!' => {
                    self.end_value();
                    self.tokens.push(Token::Operator(OperatorToken::Not));
                }
                '[' => {
                    self.end_value();
                    self.tokens.push(Token::ListOpen);
                }
                ']' => {
                    self.end_value();
                    self.tokens.push(Token::ListClose);
                }
                '(' => {
                    self.end_value();
                    self.tokens.push(Token::ParenOpen);
                }
                ')' => {
                    self.end_value();
                    self.tokens.push(Token::ParenClose);
                }
                '^' => {
                    self.end_value();
                    self.tokens.push(Token::Weight);
                }
                _ => {
                    self.current_value.push(ch);
                }
            }
        }

        self.end_value();
        self.tokens
    }
}

#[derive(Debug, Error)]
pub enum FilterParseError {
    #[error("found ^ weight operator with no following weight value")]
    MissingWeightValue,
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unexpected non integer weight value: {0}")]
    InvalidWeightValue(ParseIntError),
    #[error("encountered weight without a filter to apply to")]
    UnexpectedWeight,
    #[error("expected filter key but got: {0}")]
    UnexpectedFilterKey(String),
    #[error("expected filter value but got nothing")]
    ExpectedValue,
    #[error("expected comma separating attribute values")]
    ExpectedComma,
    #[error("failed to parse category: {0}")]
    InvalidCategory(CategoryError),
    #[error("unknown item rarity: {0}")]
    InvalidRarity(String),
    #[error("invalid item name: {0}")]
    InvalidItemName(uuid::Error),
    #[error("encountered unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("operation is missing left hand side")]
    OperationMissingLeft,
    #[error("operation is missing right hand side")]
    OperationMissingRight,
    #[error("list of filter values was empty")]
    EmptyList,
}

pub fn parse_filter(value: &str) -> Result<Filter, FilterParseError> {
    let tokens = Tokenizer::tokenize_value(value);
    Parser::parse_tokens(&tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

enum ValueSet<'a> {
    /// Singular value match
    Value(&'a str),
    /// Match for any of the values in the set
    Values(Vec<&'a str>),
}

impl<'a> ValueSet<'a> {
    /// Parses the values set using  the filter parse function, combines the
    /// results using the OR filter operation
    pub fn parse_as_filter<F>(&self, parse_filter: F) -> Result<Filter, FilterParseError>
    where
        F: Fn(&'a str) -> Result<Filter, FilterParseError>,
    {
        match self {
            ValueSet::Value(value) => parse_filter(value),
            ValueSet::Values(values) => {
                let mut filter: Option<Filter> = None;

                for value in values {
                    let new_filter = parse_filter(value)?;
                    if let Some(current_filter) = filter {
                        filter = Some(current_filter.or(new_filter));
                    } else {
                        filter = Some(new_filter);
                    }
                }

                filter.ok_or(FilterParseError::EmptyList)
            }
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_tokens(tokens: &[Token]) -> Result<Filter, FilterParseError> {
        let mut parser = Parser { tokens, index: 0 };
        parser.parse_filter()
    }

    fn next_token(&mut self) -> Option<&'a Token> {
        let token = self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }

    fn back(&mut self) {
        if self.index > 1 {
            self.index -= 1;
        }
    }

    fn has_next_token(&mut self) -> bool {
        self.index + 1 < self.tokens.len()
    }

    fn collect_value_token(&mut self) -> Result<ValueSet<'_>, FilterParseError> {
        match self.next_token().ok_or(FilterParseError::ExpectedValue)? {
            Token::Value(value) => return Ok(ValueSet::Value(value)),
            Token::ListOpen => {} // Continue to list parsing
            _ => return Err(FilterParseError::UnexpectedToken),
        }

        let mut values = Vec::new();

        while let Some(token) = self.next_token() {
            match token {
                Token::Value(value) => {
                    values.push(value.as_str());
                    continue;
                }
                Token::Operator(OperatorToken::Comma) => continue,
                Token::ListClose => return Ok(ValueSet::Values(values)),
                _ => return Err(FilterParseError::UnexpectedToken),
            }
        }

        // Reached the end without closing the list
        Err(FilterParseError::UnexpectedEndOfInput)
    }

    fn parse_value_token(&mut self, value: &str) -> Result<Filter, FilterParseError> {
        match value.trim() {
            "Category" => {
                let value_set = self.collect_value_token()?;
                value_set.parse_as_filter(|value| {
                    Category::from_str(value)
                        .map(Filter::Category)
                        .map_err(FilterParseError::InvalidCategory)
                })
            }
            "Rarity" => {
                let value_set = self.collect_value_token()?;

                value_set.parse_as_filter(|value| {
                    ItemRarity::from_str(value)
                        .map(Filter::Rarity)
                        .map_err(|_| FilterParseError::InvalidRarity(value.to_string()))
                })
            }
            "Name" => {
                let value_set = self.collect_value_token()?;
                value_set.parse_as_filter(|value| {
                    ItemName::from_str(value)
                        .map(Filter::Named)
                        .map_err(FilterParseError::InvalidItemName)
                })
            }
            "Attribute" => {
                let key = self
                    .next_token()
                    .ok_or(FilterParseError::ExpectedValue)?
                    .as_value()
                    .ok_or(FilterParseError::UnexpectedToken)?;

                if !matches!(
                    self.next_token(),
                    Some(Token::Operator(OperatorToken::Comma))
                ) {
                    return Err(FilterParseError::ExpectedComma);
                }

                let value_set = self.collect_value_token()?;
                value_set.parse_as_filter(|value| Ok(Filter::attribute(key, value)))
            }
            _ => Err(FilterParseError::UnexpectedFilterKey(value.to_string())),
        }
    }

    fn parse_expression(&mut self, expression: bool) -> Result<Filter, FilterParseError> {
        let mut stack: Vec<Filter> = Vec::new();
        let mut current_op: Option<OperatorToken> = None;

        while let Some(token) = self.next_token() {
            match token {
                Token::Value(value) => {
                    let filter = self.parse_value_token(value)?;
                    stack.push(filter);
                }

                //
                Token::Operator(operator) => {
                    current_op = Some(*operator);
                    continue;
                }

                //
                Token::ParenOpen => {
                    if !self.has_next_token() {
                        return Err(FilterParseError::UnexpectedEndOfInput);
                    }

                    let filter = self.parse_expression(true)?;
                    stack.push(filter);

                    // Should end with a closing parenthesis
                    if !matches!(self.next_token(), Some(Token::ParenClose)) {
                        return Err(FilterParseError::UnexpectedToken);
                    }
                }

                //
                Token::ParenClose => {
                    // Should not encounter a closing parenthesis at a depth of zero
                    // this is an unopened parenthesis
                    if !expression {
                        return Err(FilterParseError::UnexpectedToken);
                    }

                    // Do not consume the closing parenthesis token
                    self.back();
                    break;
                }

                // Should never see list tokens at the top level
                Token::ListClose | Token::ListOpen => {
                    return Err(FilterParseError::UnexpectedToken);
                }

                Token::Weight => {
                    let weight_value = self
                        .next_token()
                        .ok_or(FilterParseError::MissingWeightValue)?
                        .as_value()
                        .ok_or(FilterParseError::UnexpectedToken)?;

                    let weight = weight_value
                        .parse::<u32>()
                        .map_err(FilterParseError::InvalidWeightValue)?;

                    // The latest filter should be used for the weight
                    let filter = stack.pop().ok_or(FilterParseError::UnexpectedWeight)?;
                    stack.push(Filter::Weighted(Box::new(filter), weight));
                }
            }

            if let Some(operation) = current_op.take() {
                match operation {
                    OperatorToken::And => {
                        let right = stack.pop().ok_or(FilterParseError::OperationMissingRight)?;
                        let left = stack.pop().ok_or(FilterParseError::OperationMissingLeft)?;
                        stack.push(Filter::And(Box::new(left), Box::new(right)))
                    }
                    OperatorToken::Or => {
                        let right = stack.pop().ok_or(FilterParseError::OperationMissingRight)?;
                        let left = stack.pop().ok_or(FilterParseError::OperationMissingLeft)?;
                        stack.push(Filter::Or(Box::new(left), Box::new(right)))
                    }
                    OperatorToken::Comma => {
                        let right = stack.pop().ok_or(FilterParseError::OperationMissingRight)?;
                        let mut left = stack.pop().ok_or(FilterParseError::OperationMissingLeft)?;

                        let filter = match &mut left {
                            // Merge into existing "many" filter
                            Filter::Many(filters) => {
                                filters.push(right);
                                left
                            }
                            // Create new many filter
                            _ => Filter::Many(vec![left, right]),
                        };

                        stack.push(filter);
                    }
                    OperatorToken::Not => {
                        let filter = stack.pop().ok_or(FilterParseError::OperationMissingRight)?;
                        stack.push(Filter::Not(Box::new(filter)));
                    }
                }
            }
        }

        stack.pop().ok_or(FilterParseError::UnexpectedEndOfInput)
    }

    fn parse_filter(&mut self) -> Result<Filter, FilterParseError> {
        let mut stack: Vec<Filter> = Vec::new();

        while self.has_next_token() {
            let filter = self.parse_expression(false)?;
            stack.push(filter);
        }

        stack.pop().ok_or(FilterParseError::UnexpectedEndOfInput)
    }
}

#[cfg(test)]
mod test {
    use crate::definitions::{
        items::{
            category::{BaseCategory, Category},
            rarity::ItemRarity,
        },
        packs::{Filter, parser::parse_filter},
    };

    #[test]
    fn test_complex_expression() {
        let expression = "!(Category=2||Category=14) && (Rarity=0)";
        let value = parse_filter(expression).unwrap();

        assert_eq!(
            value,
            Filter::categories([
                Category::Base(BaseCategory::WeaponMods),
                Category::Base(BaseCategory::WeaponModsEnhanced)
            ])
            .not()
            .and(Filter::rarities([ItemRarity::Common]))
        );
    }
    #[test]
    fn test_list_expression() {
        let expression = "!(Category=[2,14]) && (Rarity=0)";
        let value = parse_filter(expression).unwrap();

        assert_eq!(
            value,
            Filter::categories([
                Category::Base(BaseCategory::WeaponMods),
                Category::Base(BaseCategory::WeaponModsEnhanced)
            ])
            .not()
            .and(Filter::rarities([ItemRarity::Common]))
        );
    }
}
