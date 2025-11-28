//! Query language for searching the music library.
//!
//! Apollo supports a simple query language for filtering tracks and albums.
//!
//! # Query Syntax
//!
//! - `artist:name` - Match artist field
//! - `album:name` - Match album field  
//! - `title:name` - Match title field
//! - `year:2020` - Match exact year
//! - `year:2020..2023` - Match year range
//! - `genre:rock` - Match genre
//! - `path:/music/` - Match path prefix
//! - Simple text searches all fields

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// A parsed query for searching the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    /// Match all items.
    All,
    /// Full-text search across all fields.
    Text(String),
    /// Match a specific field.
    Field { field: Field, value: String },
    /// Match a year range.
    YearRange { start: i32, end: i32 },
    /// Combine queries with AND.
    And(Vec<Query>),
    /// Combine queries with OR.
    Or(Vec<Query>),
    /// Negate a query.
    Not(Box<Query>),
}

/// Fields that can be queried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Field {
    Artist,
    AlbumArtist,
    Album,
    Title,
    Year,
    Genre,
    Path,
}

impl Query {
    /// Parse a query string into a Query.
    ///
    /// # Errors
    ///
    /// Returns an error if the query syntax is invalid.
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(Self::All);
        }

        // Simple implementation: check for field:value patterns
        if let Some((field, value)) = input.split_once(':') {
            let field = match field.to_lowercase().as_str() {
                "artist" => Field::Artist,
                "albumartist" | "album_artist" => Field::AlbumArtist,
                "album" => Field::Album,
                "title" => Field::Title,
                "year" => Field::Year,
                "genre" => Field::Genre,
                "path" => Field::Path,
                _ => return Err(Error::InvalidQuery(format!("unknown field: {field}"))),
            };

            // Check for year range
            if field == Field::Year
                && value.contains("..")
                && let Some((start, end)) = value.split_once("..")
            {
                let start: i32 = start
                    .parse()
                    .map_err(|_| Error::InvalidQuery(format!("invalid year: {start}")))?;
                let end: i32 = end
                    .parse()
                    .map_err(|_| Error::InvalidQuery(format!("invalid year: {end}")))?;
                return Ok(Self::YearRange { start, end });
            }

            Ok(Self::Field {
                field,
                value: value.to_string(),
            })
        } else {
            // Plain text search
            Ok(Self::Text(input.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parse_empty_query() {
        let query = Query::parse("").unwrap();
        assert!(matches!(query, Query::All));
    }

    #[test]
    fn parse_text_query() {
        let query = Query::parse("beatles").unwrap();
        assert!(matches!(query, Query::Text(ref s) if s == "beatles"));
    }

    #[test]
    fn parse_field_query() {
        let query = Query::parse("artist:Beatles").unwrap();
        assert!(matches!(
            query,
            Query::Field { field: Field::Artist, ref value } if value == "Beatles"
        ));
    }

    #[test]
    fn parse_year_range() {
        let query = Query::parse("year:2020..2023").unwrap();
        assert!(matches!(
            query,
            Query::YearRange {
                start: 2020,
                end: 2023
            }
        ));
    }

    /// Strategy for generating valid field names.
    fn field_name_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("artist"),
            Just("albumartist"),
            Just("album_artist"),
            Just("album"),
            Just("title"),
            Just("year"),
            Just("genre"),
            Just("path"),
        ]
    }

    /// Strategy for generating search values without colons and without leading/trailing spaces.
    fn search_value_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_/-]{1,30}"
    }

    proptest! {
        /// Test that whitespace-only queries parse as All.
        #[test]
        fn whitespace_parses_as_all(s in "[ \t\n\r]*") {
            let query = Query::parse(&s).expect("whitespace should parse");
            prop_assert!(matches!(query, Query::All));
        }

        /// Test that text without colons parses as Text query.
        #[test]
        fn text_without_colon_parses_as_text(s in "[a-zA-Z0-9 _-]{1,50}") {
            // Only test if there's no colon
            if !s.contains(':') {
                let query = Query::parse(&s).expect("text should parse");
                match query {
                    Query::Text(text) => prop_assert_eq!(text, s.trim()),
                    Query::All => prop_assert!(s.trim().is_empty()),
                    _ => prop_assert!(false, "expected Text or All"),
                }
            }
        }

        /// Test that valid field:value queries parse correctly.
        #[test]
        fn valid_field_value_parses(
            field in field_name_strategy(),
            value in search_value_strategy(),
        ) {
            let input = format!("{field}:{value}");
            let query = Query::parse(&input).expect("valid field:value should parse");

            match query {
                Query::Field { value: parsed_value, .. } => {
                    prop_assert_eq!(parsed_value, value);
                }
                Query::YearRange { .. } => {
                    // This is valid for year field with ".." in value
                    prop_assert!(field == "year" && value.contains(".."));
                }
                _ => prop_assert!(false, "expected Field or YearRange"),
            }
        }

        /// Test that year range queries parse correctly.
        #[test]
        fn year_range_parses(
            start in 1900i32..2100i32,
            end in 1900i32..2100i32,
        ) {
            let input = format!("year:{start}..{end}");
            let query = Query::parse(&input).expect("year range should parse");

            match query {
                Query::YearRange { start: s, end: e } => {
                    prop_assert_eq!(s, start);
                    prop_assert_eq!(e, end);
                }
                _ => prop_assert!(false, "expected YearRange"),
            }
        }

        /// Test that invalid field names produce errors.
        #[test]
        fn invalid_field_produces_error(
            field in "[a-z]{1,10}",
            value in search_value_strategy(),
        ) {
            // Only test if the field is not a valid field name
            let valid_fields = ["artist", "albumartist", "album_artist", "album", "title", "year", "genre", "path"];
            if !valid_fields.contains(&field.as_str()) {
                let input = format!("{field}:{value}");
                let result = Query::parse(&input);
                prop_assert!(result.is_err(), "invalid field should produce error");
            }
        }

        /// Test that Query serialization roundtrips correctly.
        #[test]
        fn query_serialization_roundtrip(
            field in field_name_strategy(),
            value in "[a-zA-Z0-9]{1,20}",
        ) {
            let input = format!("{field}:{value}");
            let query = Query::parse(&input).expect("should parse");

            let json = serde_json::to_string(&query).expect("serialization should succeed");
            let deserialized: Query = serde_json::from_str(&json).expect("deserialization should succeed");

            // Compare serialized forms since Query doesn't implement PartialEq
            let json2 = serde_json::to_string(&deserialized).expect("re-serialization should succeed");
            prop_assert_eq!(json, json2);
        }

        /// Test that parsing is idempotent for the string representation.
        #[test]
        fn parsing_preserves_value(value in "[a-zA-Z0-9]{1,20}") {
            let input = format!("artist:{value}");
            let query = Query::parse(&input).expect("should parse");

            if let Query::Field { value: parsed, .. } = query {
                prop_assert_eq!(parsed, value);
            } else {
                prop_assert!(false, "expected Field");
            }
        }
    }
}
