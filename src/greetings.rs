//! Greeting data for the demo `greet` tool.
//!
//! Maps a handful of languages to a greeting word. Lookups are case-insensitive
//! and also accept common alternate spellings / ISO codes (e.g. `fr` or
//! `Français` for French). Add a language by adding a row to [`GREETINGS`]
//! (and, optionally, an alias to [`ALIASES`]).

/// Canonical language name -> greeting word, in definition order.
pub const GREETINGS: &[(&str, &str)] = &[
    ("english", "Hello"),
    ("spanish", "Hola"),
    ("french", "Bonjour"),
    ("german", "Hallo"),
    ("italian", "Ciao"),
    ("portuguese", "Olá"),
    ("japanese", "こんにちは (Konnichiwa)"),
    ("hawaiian", "Aloha"),
];

/// Language used when the caller doesn't specify one.
pub const DEFAULT_LANGUAGE: &str = "english";

/// Alternate spellings / ISO codes -> canonical language name.
pub const ALIASES: &[(&str, &str)] = &[
    ("en", "english"),
    ("es", "spanish"),
    ("espanol", "spanish"),
    ("español", "spanish"),
    ("fr", "french"),
    ("francais", "french"),
    ("français", "french"),
    ("de", "german"),
    ("deutsch", "german"),
    ("it", "italian"),
    ("italiano", "italian"),
    ("pt", "portuguese"),
    ("portugues", "portuguese"),
    ("português", "portuguese"),
    ("ja", "japanese"),
    ("jp", "japanese"),
    ("nihongo", "japanese"),
    ("haw", "hawaiian"),
];

/// The languages this server knows how to greet in, in definition order.
pub fn languages() -> Vec<&'static str> {
    GREETINGS.iter().map(|(name, _)| *name).collect()
}

/// Look up the greeting word for a canonical language name.
fn greeting_word(canonical: &str) -> Option<&'static str> {
    GREETINGS
        .iter()
        .find(|(name, _)| *name == canonical)
        .map(|(_, word)| *word)
}

/// The result of a successful [`greet`] call: `{ language, greeting, message }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Greeting {
    pub language: String,
    pub greeting: String,
    pub message: String,
}

/// Return the canonical language name for `language`.
///
/// Accepts a canonical name, an alias, or an ISO code (case-insensitive). A
/// `None` or blank value yields the default (English). Returns `Err` with a
/// message listing the supported set for an unknown language.
pub fn resolve_language(language: Option<&str>) -> Result<&'static str, String> {
    let key = match language {
        Some(v) if !v.trim().is_empty() => v.trim().to_lowercase(),
        _ => return Ok(DEFAULT_LANGUAGE),
    };
    if let Some((name, _)) = GREETINGS.iter().find(|(name, _)| *name == key) {
        return Ok(name);
    }
    if let Some((_, canonical)) = ALIASES.iter().find(|(alias, _)| *alias == key) {
        return Ok(canonical);
    }
    Err(format!(
        "unknown language '{}'; supported: {}",
        language.unwrap_or(""),
        languages().join(", ")
    ))
}

/// Build a greeting record for `language` (default English).
///
/// Pass an optional `name` to personalize the message (e.g. `"Bonjour,
/// Alice!"`). Returns `{ language, greeting, message }` or an error for an
/// unknown language.
pub fn greet(language: Option<&str>, name: Option<&str>) -> Result<Greeting, String> {
    let canonical = resolve_language(language)?;
    let word = greeting_word(canonical).expect("resolved language always has a greeting");
    let message = match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => format!("{word}, {n}!"),
        None => format!("{word}!"),
    };
    Ok(Greeting {
        language: canonical.to_string(),
        greeting: word.to_string(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_language_is_english() {
        assert_eq!(DEFAULT_LANGUAGE, "english");
        assert_eq!(resolve_language(None).unwrap(), "english");
        assert_eq!(resolve_language(Some("")).unwrap(), "english");
        assert_eq!(resolve_language(Some("   ")).unwrap(), "english");
    }

    #[test]
    fn resolve_is_case_insensitive() {
        assert_eq!(resolve_language(Some("French")).unwrap(), "french");
        assert_eq!(resolve_language(Some("  SPANISH  ")).unwrap(), "spanish");
    }

    #[test]
    fn resolve_accepts_aliases_and_codes() {
        for (value, expected) in [
            ("fr", "french"),
            ("es", "spanish"),
            ("jp", "japanese"),
            ("Français", "french"),
        ] {
            assert_eq!(resolve_language(Some(value)).unwrap(), expected);
        }
    }

    #[test]
    fn resolve_unknown_language_errors() {
        let err = resolve_language(Some("klingon")).unwrap_err();
        // The error lists the supported languages so a caller can recover.
        assert!(err.contains("supported"));
        assert!(err.contains("english"));
    }

    #[test]
    fn greet_defaults_to_english() {
        let g = greet(None, None).unwrap();
        assert_eq!(g.language, "english");
        assert_eq!(g.greeting, "Hello");
        assert_eq!(g.message, "Hello!");
    }

    #[test]
    fn greet_in_french() {
        let g = greet(Some("French"), None).unwrap();
        assert_eq!(g.language, "french");
        assert_eq!(g.greeting, "Bonjour");
        assert_eq!(g.message, "Bonjour!");
    }

    #[test]
    fn greet_personalized() {
        let g = greet(Some("french"), Some("Alice")).unwrap();
        assert_eq!(g.message, "Bonjour, Alice!");
    }
}
