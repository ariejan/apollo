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
}
