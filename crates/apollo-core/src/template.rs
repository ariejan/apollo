//! Path templates for organizing music files.
//!
//! Apollo uses template strings to determine how files should be organized.
//! Templates support variable substitution and functions for text transformation.
//!
//! # Template Syntax
//!
//! ## Variables
//!
//! Variables are prefixed with `$` and can be written with or without braces:
//!
//! - `$artist` - Simple variable
//! - `${artist}` - Variable with explicit braces (useful when followed by text)
//!
//! ## Available Variables
//!
//! - `$artist` - Track artist
//! - `$album_artist` - Album artist (falls back to artist if not set)
//! - `$album` - Album title
//! - `$title` - Track title
//! - `$track` - Track number (zero-padded to 2 digits)
//! - `$disc` - Disc number
//! - `$year` - Release year
//! - `$genre` - First genre (if any)
//! - `$ext` - File extension (without dot)
//!
//! ## Functions
//!
//! Functions use the syntax `%func{arg1,arg2,...}`:
//!
//! - `%upper{text}` - Convert to uppercase
//! - `%lower{text}` - Convert to lowercase
//! - `%title{text}` - Convert to title case
//! - `%left{text,n}` - Take first n characters
//! - `%right{text,n}` - Take last n characters
//! - `%if{condition,then}` - Output `then` if condition is non-empty
//! - `%if{condition,then,else}` - Output `then` or `else` based on condition
//! - `%first{text,text,...}` - Return first non-empty value
//! - `%replace{text,from,to}` - Replace occurrences
//! - `%sanitize{text}` - Remove/replace filesystem-unsafe characters
//!
//! # Examples
//!
//! ```
//! use apollo_core::template::{PathTemplate, TemplateContext};
//! use std::path::PathBuf;
//!
//! let template = PathTemplate::parse("$artist/$album/$track - $title").unwrap();
//!
//! let mut ctx = TemplateContext::new();
//! ctx.set("artist", "Queen");
//! ctx.set("album", "A Night at the Opera");
//! ctx.set("track", "11");
//! ctx.set("title", "Bohemian Rhapsody");
//! ctx.set("ext", "mp3");
//!
//! let path = template.render(&ctx).unwrap();
//! assert_eq!(path, PathBuf::from("Queen/A Night at the Opera/11 - Bohemian Rhapsody"));
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::Error;
use crate::metadata::Track;

/// A parsed path template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTemplate {
    /// The original template string.
    source: String,
    /// Parsed template parts.
    parts: Vec<TemplatePart>,
}

/// A part of a parsed template.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplatePart {
    /// Literal text.
    Literal(String),
    /// Variable substitution.
    Variable(String),
    /// Function call.
    Function {
        name: String,
        args: Vec<TemplateExpr>,
    },
}

/// An expression that can appear in function arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplateExpr {
    /// Literal text.
    Literal(String),
    /// Variable reference.
    Variable(String),
    /// Nested function call.
    Function {
        name: String,
        args: Vec<TemplateExpr>,
    },
}

/// Context for template rendering, containing variable values.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable value.
    pub fn set(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    /// Get a variable value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(String::as_str)
    }

    /// Create a context from a Track.
    #[must_use]
    pub fn from_track(track: &Track) -> Self {
        let mut ctx = Self::new();

        ctx.set("artist", &track.artist);
        ctx.set(
            "album_artist",
            track.album_artist.as_deref().unwrap_or(&track.artist),
        );
        ctx.set("title", &track.title);

        if let Some(album) = &track.album_title {
            ctx.set("album", album);
        }

        if let Some(num) = track.track_number {
            ctx.set("track", &format!("{num:02}"));
        }

        if let Some(num) = track.disc_number {
            ctx.set("disc", &format!("{num}"));
        }

        if let Some(year) = track.year {
            ctx.set("year", &format!("{year}"));
        }

        if let Some(genre) = track.genres.first() {
            ctx.set("genre", genre);
        }

        // Extract extension from path
        if let Some(ext) = track.path.extension().and_then(|e| e.to_str()) {
            ctx.set("ext", ext);
        }

        ctx
    }
}

impl PathTemplate {
    /// Parse a template string.
    ///
    /// # Errors
    ///
    /// Returns an error if the template syntax is invalid.
    pub fn parse(template: &str) -> Result<Self, Error> {
        let parts = parse_template(template)?;
        Ok(Self {
            source: template.to_string(),
            parts,
        })
    }

    /// Get the original template string.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Render the template with the given context.
    ///
    /// # Errors
    ///
    /// Returns an error if a required variable is missing or a function fails.
    pub fn render(&self, ctx: &TemplateContext) -> Result<PathBuf, Error> {
        let mut result = String::new();

        for part in &self.parts {
            let value = render_part(part, ctx)?;
            result.push_str(&value);
        }

        // Clean up the path: remove leading/trailing slashes, collapse multiple slashes
        let result = result
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/");

        Ok(PathBuf::from(result))
    }

    /// Render the template and include the file extension.
    ///
    /// This is a convenience method that appends `.$ext` if not already in the template.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render_with_extension(&self, ctx: &TemplateContext) -> Result<PathBuf, Error> {
        let path = self.render(ctx)?;

        // Check if template already includes extension
        // Allow literal_string_with_formatting_args because this is actual template syntax
        #[allow(clippy::literal_string_with_formatting_args)]
        let has_ext = self.source.contains("$ext") || self.source.contains("${ext}");
        if has_ext {
            return Ok(path);
        }

        // Append extension if available
        if let Some(ext) = ctx.get("ext") {
            let path_str = path.to_string_lossy();
            return Ok(PathBuf::from(format!("{path_str}.{ext}")));
        }

        Ok(path)
    }
}

/// Parse a template string into parts.
fn parse_template(template: &str) -> Result<Vec<TemplatePart>, Error> {
    let mut parts = Vec::new();
    let mut chars = template.chars().peekable();
    let mut literal = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '$' => {
                // Flush literal
                if !literal.is_empty() {
                    parts.push(TemplatePart::Literal(std::mem::take(&mut literal)));
                }

                // Parse variable
                let var_name = parse_variable_name(&mut chars)?;
                parts.push(TemplatePart::Variable(var_name));
            }
            '%' => {
                // Flush literal
                if !literal.is_empty() {
                    parts.push(TemplatePart::Literal(std::mem::take(&mut literal)));
                }

                // Parse function
                let (name, args) = parse_function(&mut chars)?;
                parts.push(TemplatePart::Function { name, args });
            }
            '\\' => {
                // Escape next character
                if let Some(next) = chars.next() {
                    literal.push(next);
                } else {
                    literal.push('\\');
                }
            }
            _ => {
                literal.push(ch);
            }
        }
    }

    // Flush remaining literal
    if !literal.is_empty() {
        parts.push(TemplatePart::Literal(literal));
    }

    Ok(parts)
}

/// Parse a variable name after `$`.
fn parse_variable_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, Error> {
    let mut name = String::new();

    // Check for braced form ${name}
    if chars.peek() == Some(&'{') {
        chars.next(); // consume '{'

        while let Some(&ch) = chars.peek() {
            if ch == '}' {
                chars.next(); // consume '}'
                break;
            }
            name.push(chars.next().unwrap());
        }

        if name.is_empty() {
            return Err(Error::Validation("Empty variable name".to_string()));
        }

        return Ok(name);
    }

    // Simple form: $name (alphanumeric + underscore)
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    if name.is_empty() {
        return Err(Error::Validation("Empty variable name after $".to_string()));
    }

    Ok(name)
}

/// Parse a function after `%`.
fn parse_function(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<(String, Vec<TemplateExpr>), Error> {
    let mut name = String::new();

    // Parse function name (alphanumeric + underscore)
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    if name.is_empty() {
        return Err(Error::Validation("Empty function name after %".to_string()));
    }

    // Expect '{'
    if chars.peek() != Some(&'{') {
        return Err(Error::Validation(format!(
            "Expected '{{' after function name '{name}'"
        )));
    }
    chars.next(); // consume '{'

    // Parse arguments
    let args = parse_function_args(chars)?;

    Ok((name, args))
}

/// Parse function arguments inside `{...}`.
fn parse_function_args(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Vec<TemplateExpr>, Error> {
    let mut args = Vec::new();
    let mut current_arg = Vec::new();
    let mut depth = 1; // We're inside one '{'

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                depth += 1;
                current_arg.push(TemplateExpr::Literal("{".to_string()));
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    // End of function
                    if !current_arg.is_empty() {
                        args.push(flatten_exprs(current_arg));
                    }
                    return Ok(args);
                }
                current_arg.push(TemplateExpr::Literal("}".to_string()));
            }
            ',' if depth == 1 => {
                // Argument separator
                args.push(flatten_exprs(std::mem::take(&mut current_arg)));
            }
            '$' => {
                // Variable in argument
                let var_name = parse_variable_name(chars)?;
                current_arg.push(TemplateExpr::Variable(var_name));
            }
            '%' => {
                // Nested function
                let (name, nested_args) = parse_function(chars)?;
                current_arg.push(TemplateExpr::Function {
                    name,
                    args: nested_args,
                });
            }
            '\\' => {
                // Escape
                if let Some(next) = chars.next() {
                    current_arg.push(TemplateExpr::Literal(next.to_string()));
                }
            }
            _ => {
                // Add to current literal or start new one
                if let Some(TemplateExpr::Literal(s)) = current_arg.last_mut() {
                    s.push(ch);
                } else {
                    current_arg.push(TemplateExpr::Literal(ch.to_string()));
                }
            }
        }
    }

    Err(Error::Validation(
        "Unclosed function: missing '}'".to_string(),
    ))
}

/// Flatten a list of expressions into a single expression.
fn flatten_exprs(exprs: Vec<TemplateExpr>) -> TemplateExpr {
    if exprs.len() == 1 {
        return exprs.into_iter().next().unwrap();
    }

    // Combine consecutive literals
    let mut result = Vec::new();
    let mut current_literal = String::new();

    for expr in exprs {
        match expr {
            TemplateExpr::Literal(s) => {
                current_literal.push_str(&s);
            }
            other => {
                if !current_literal.is_empty() {
                    result.push(TemplateExpr::Literal(std::mem::take(&mut current_literal)));
                }
                result.push(other);
            }
        }
    }

    if !current_literal.is_empty() {
        result.push(TemplateExpr::Literal(current_literal));
    }

    if result.len() == 1 {
        result.into_iter().next().unwrap()
    } else {
        // Wrap in a concat function (implicit)
        TemplateExpr::Function {
            name: "_concat".to_string(),
            args: result,
        }
    }
}

/// Render a template part.
fn render_part(part: &TemplatePart, ctx: &TemplateContext) -> Result<String, Error> {
    match part {
        TemplatePart::Literal(s) => Ok(s.clone()),
        TemplatePart::Variable(name) => ctx
            .get(name)
            .map(String::from)
            .ok_or_else(|| Error::Validation(format!("Unknown variable: ${name}"))),
        TemplatePart::Function { name, args } => render_function(name, args, ctx),
    }
}

/// Render a template expression.
fn render_expr(expr: &TemplateExpr, ctx: &TemplateContext) -> Result<String, Error> {
    match expr {
        TemplateExpr::Literal(s) => Ok(s.clone()),
        TemplateExpr::Variable(name) => ctx
            .get(name)
            .map(String::from)
            .ok_or_else(|| Error::Validation(format!("Unknown variable: ${name}"))),
        TemplateExpr::Function { name, args } => render_function(name, args, ctx),
    }
}

/// Render a function call.
fn render_function(
    name: &str,
    args: &[TemplateExpr],
    ctx: &TemplateContext,
) -> Result<String, Error> {
    match name {
        "_concat" => {
            // Internal: concatenate all arguments
            let mut result = String::new();
            for arg in args {
                result.push_str(&render_expr(arg, ctx)?);
            }
            Ok(result)
        }
        "upper" => {
            require_args(name, args, 1)?;
            Ok(render_expr(&args[0], ctx)?.to_uppercase())
        }
        "lower" => {
            require_args(name, args, 1)?;
            Ok(render_expr(&args[0], ctx)?.to_lowercase())
        }
        "title" => {
            require_args(name, args, 1)?;
            Ok(to_title_case(&render_expr(&args[0], ctx)?))
        }
        "left" => {
            require_args(name, args, 2)?;
            let text = render_expr(&args[0], ctx)?;
            let n: usize = render_expr(&args[1], ctx)?
                .parse()
                .map_err(|_| Error::Validation("left: second argument must be a number".into()))?;
            Ok(text.chars().take(n).collect())
        }
        "right" => {
            require_args(name, args, 2)?;
            let text = render_expr(&args[0], ctx)?;
            let n: usize = render_expr(&args[1], ctx)?
                .parse()
                .map_err(|_| Error::Validation("right: second argument must be a number".into()))?;
            let chars: Vec<char> = text.chars().collect();
            let start = chars.len().saturating_sub(n);
            Ok(chars[start..].iter().collect())
        }
        "if" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(Error::Validation(
                    "if: requires 2 or 3 arguments".to_string(),
                ));
            }
            let condition = render_expr(&args[0], ctx)?;
            if !condition.is_empty() {
                render_expr(&args[1], ctx)
            } else if args.len() == 3 {
                render_expr(&args[2], ctx)
            } else {
                Ok(String::new())
            }
        }
        "first" => {
            for arg in args {
                let value = render_expr(arg, ctx)?;
                if !value.is_empty() {
                    return Ok(value);
                }
            }
            Ok(String::new())
        }
        "replace" => {
            require_args(name, args, 3)?;
            let text = render_expr(&args[0], ctx)?;
            let from = render_expr(&args[1], ctx)?;
            let to = render_expr(&args[2], ctx)?;
            Ok(text.replace(&from, &to))
        }
        "sanitize" => {
            require_args(name, args, 1)?;
            let text = render_expr(&args[0], ctx)?;
            Ok(sanitize_path_component(&text))
        }
        "asciify" => {
            require_args(name, args, 1)?;
            let text = render_expr(&args[0], ctx)?;
            Ok(asciify(&text))
        }
        "padnum" => {
            require_args(name, args, 2)?;
            let text = render_expr(&args[0], ctx)?;
            let width: usize = render_expr(&args[1], ctx)?.parse().map_err(|_| {
                Error::Validation("padnum: second argument must be a number".into())
            })?;
            // Try to parse as number and pad
            Ok(text
                .parse::<u32>()
                .map_or_else(|_| text.clone(), |num| format!("{num:0>width$}")))
        }
        _ => Err(Error::Validation(format!("Unknown function: %{name}"))),
    }
}

/// Check that a function has the required number of arguments.
fn require_args(name: &str, args: &[TemplateExpr], count: usize) -> Result<(), Error> {
    if args.len() != count {
        return Err(Error::Validation(format!(
            "{name}: requires {count} argument(s), got {}",
            args.len()
        )));
    }
    Ok(())
}

/// Convert a string to title case.
fn to_title_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch.is_whitespace() {
            capitalize_next = true;
            result.push(ch);
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(ch.to_lowercase());
        }
    }

    result
}

/// Sanitize a string for use as a path component.
///
/// This removes or replaces characters that are invalid in file paths.
#[must_use]
pub fn sanitize_path_component(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for ch in s.chars() {
        match ch {
            // Replace path separators with space
            '/' | '\\' => result.push(' '),
            // Remove null bytes and other control characters
            '\0'..='\x1f' | '\x7f' => {}
            // Replace other problematic characters
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => result.push('_'),
            // Keep everything else
            _ => result.push(ch),
        }
    }

    // Trim leading/trailing whitespace and dots (Windows doesn't like trailing dots)
    let result = result.trim().trim_end_matches('.');

    // Avoid empty result
    if result.is_empty() {
        return "_".to_string();
    }

    result.to_string()
}

/// Convert a string to ASCII, removing or replacing non-ASCII characters.
fn asciify(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for ch in s.chars() {
        if ch.is_ascii() {
            result.push(ch);
        } else {
            // Try to transliterate common characters
            match ch {
                'ä' | 'á' | 'à' | 'â' | 'ã' => result.push('a'),
                'Ä' | 'Á' | 'À' | 'Â' | 'Ã' => result.push('A'),
                'ë' | 'é' | 'è' | 'ê' => result.push('e'),
                'Ë' | 'É' | 'È' | 'Ê' => result.push('E'),
                'ï' | 'í' | 'ì' | 'î' => result.push('i'),
                'Ï' | 'Í' | 'Ì' | 'Î' => result.push('I'),
                'ö' | 'ó' | 'ò' | 'ô' | 'õ' | 'ø' => result.push('o'),
                'Ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ø' => result.push('O'),
                'ü' | 'ú' | 'ù' | 'û' => result.push('u'),
                'Ü' | 'Ú' | 'Ù' | 'Û' => result.push('U'),
                'ñ' => result.push('n'),
                'Ñ' => result.push('N'),
                'ç' => result.push('c'),
                'Ç' => result.push('C'),
                'ß' => result.push_str("ss"),
                'æ' => result.push_str("ae"),
                'Æ' => result.push_str("AE"),
                'œ' => result.push_str("oe"),
                'Œ' => result.push_str("OE"),
                'ý' | 'ÿ' => result.push('y'),
                'Ý' | 'Ÿ' => result.push('Y'),
                '–' | '—' => result.push('-'),
                '\u{2018}' | '\u{2019}' => result.push('\''), // ' and '
                '\u{201C}' | '\u{201D}' => result.push('"'),  // " and "
                '…' => result.push_str("..."),
                _ => {} // Drop other non-ASCII
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_variable() {
        let template = PathTemplate::parse("$artist/$album").unwrap();
        assert_eq!(template.parts.len(), 3);
    }

    #[test]
    fn test_parse_braced_variable() {
        let template = PathTemplate::parse("${artist}s Album").unwrap();
        assert_eq!(template.parts.len(), 2);
    }

    #[test]
    fn test_parse_function() {
        let template = PathTemplate::parse("%upper{$artist}").unwrap();
        assert_eq!(template.parts.len(), 1);
        assert!(matches!(template.parts[0], TemplatePart::Function { .. }));
    }

    #[test]
    fn test_render_simple() {
        let template = PathTemplate::parse("$artist/$album/$track - $title").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");
        ctx.set("album", "A Night at the Opera");
        ctx.set("track", "11");
        ctx.set("title", "Bohemian Rhapsody");

        let path = template.render(&ctx).unwrap();
        assert_eq!(
            path,
            PathBuf::from("Queen/A Night at the Opera/11 - Bohemian Rhapsody")
        );
    }

    #[test]
    fn test_render_upper() {
        let template = PathTemplate::parse("%upper{$artist}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("QUEEN"));
    }

    #[test]
    fn test_render_lower() {
        let template = PathTemplate::parse("%lower{$artist}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "QUEEN");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("queen"));
    }

    #[test]
    fn test_render_title() {
        let template = PathTemplate::parse("%title{$title}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("title", "bohemian rhapsody");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Bohemian Rhapsody"));
    }

    #[test]
    fn test_render_if_true() {
        let template = PathTemplate::parse("%if{$album,$album,Unknown}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("album", "Greatest Hits");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Greatest Hits"));
    }

    #[test]
    fn test_render_if_false() {
        let template = PathTemplate::parse("%if{$album,$album,Unknown}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("album", "");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Unknown"));
    }

    #[test]
    fn test_render_first() {
        let template = PathTemplate::parse("%first{$album_artist,$artist}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("album_artist", "");
        ctx.set("artist", "Queen");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Queen"));
    }

    #[test]
    fn test_render_left() {
        let template = PathTemplate::parse("%left{$artist,1}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Q"));
    }

    #[test]
    fn test_render_replace() {
        let template = PathTemplate::parse("%replace{$title, ,_}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("title", "Bohemian Rhapsody");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Bohemian_Rhapsody"));
    }

    #[test]
    fn test_render_sanitize() {
        let template = PathTemplate::parse("%sanitize{$title}").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("title", "What's Up?");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("What's Up_"));
    }

    #[test]
    fn test_render_complex() {
        let template =
            PathTemplate::parse("%upper{%left{$artist,1}}/$artist/%padnum{$track,2} - $title")
                .unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");
        ctx.set("track", "5");
        ctx.set("title", "I'm In Love With My Car");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Q/Queen/05 - I'm In Love With My Car"));
    }

    #[test]
    fn test_render_with_extension() {
        let template = PathTemplate::parse("$artist/$title").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");
        ctx.set("title", "Bohemian Rhapsody");
        ctx.set("ext", "mp3");

        let path = template.render_with_extension(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Queen/Bohemian Rhapsody.mp3"));
    }

    #[test]
    fn test_render_with_extension_already_in_template() {
        let template = PathTemplate::parse("$artist/$title.$ext").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");
        ctx.set("title", "Bohemian Rhapsody");
        ctx.set("ext", "mp3");

        let path = template.render_with_extension(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("Queen/Bohemian Rhapsody.mp3"));
    }

    #[test]
    fn test_asciify() {
        assert_eq!(asciify("Motörhead"), "Motorhead");
        assert_eq!(asciify("Björk"), "Bjork");
        assert_eq!(asciify("Sigur Rós"), "Sigur Ros");
        assert_eq!(asciify("naïve"), "naive");
        assert_eq!(asciify("Ænima"), "AEnima");
    }

    #[test]
    fn test_sanitize_path_component() {
        assert_eq!(sanitize_path_component("Hello/World"), "Hello World");
        assert_eq!(sanitize_path_component("What?"), "What_");
        assert_eq!(sanitize_path_component("AC/DC"), "AC DC");
        assert_eq!(sanitize_path_component("Test..."), "Test");
        assert_eq!(sanitize_path_component(""), "_");
    }

    #[test]
    fn test_from_track() {
        use std::time::Duration;

        let mut track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Bohemian Rhapsody".to_string(),
            "Queen".to_string(),
            Duration::from_secs(354),
        );
        track.album_title = Some("A Night at the Opera".to_string());
        track.track_number = Some(11);
        track.year = Some(1975);
        track.genres = vec!["Rock".to_string()];

        let ctx = TemplateContext::from_track(&track);

        assert_eq!(ctx.get("artist"), Some("Queen"));
        assert_eq!(ctx.get("title"), Some("Bohemian Rhapsody"));
        assert_eq!(ctx.get("album"), Some("A Night at the Opera"));
        assert_eq!(ctx.get("track"), Some("11"));
        assert_eq!(ctx.get("year"), Some("1975"));
        assert_eq!(ctx.get("genre"), Some("Rock"));
        assert_eq!(ctx.get("ext"), Some("mp3"));
    }

    #[test]
    fn test_escape() {
        let template = PathTemplate::parse(r"\$artist").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");

        let path = template.render(&ctx).unwrap();
        assert_eq!(path, PathBuf::from("$artist"));
    }

    #[test]
    fn test_missing_variable_error() {
        let template = PathTemplate::parse("$artist/$unknown").unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("artist", "Queen");

        let result = template.render(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_function_error() {
        let template = PathTemplate::parse("%unknown{test}").unwrap();
        let ctx = TemplateContext::new();

        let result = template.render(&ctx);
        assert!(result.is_err());
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_sanitize_never_empty(s in ".*") {
            let result = sanitize_path_component(&s);
            prop_assert!(!result.is_empty());
        }

        #[test]
        fn test_sanitize_no_forbidden_chars(s in ".*") {
            let result = sanitize_path_component(&s);
            prop_assert!(!result.contains('/'));
            prop_assert!(!result.contains('\\'));
            prop_assert!(!result.contains(':'));
            prop_assert!(!result.contains('*'));
            prop_assert!(!result.contains('?'));
            prop_assert!(!result.contains('"'));
            prop_assert!(!result.contains('<'));
            prop_assert!(!result.contains('>'));
            prop_assert!(!result.contains('|'));
        }

        #[test]
        fn test_asciify_is_ascii(s in ".*") {
            let result = asciify(&s);
            prop_assert!(result.is_ascii());
        }
    }
}
