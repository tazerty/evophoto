//! Evora build (patch E9): the display overlay catalog (`overlay.txt`). Third-party product terms
//! are renamed where the UI shows text, for every language and for English, after the language
//! catalog; command ids, catalog keys, the engine crates and saved files keep upstream's wording.
//!
//! The lookups return borrowed strings, so a rewritten string is built once and kept for the rest
//! of the run (the strings that need it come from the catalogs and a few labels: a small, bounded
//! set). The CI gate applies the same file to the catalogs and checks that no sensitive term is
//! left on screen.
//!
//! Patch EVORA-LUM-114 (decision Q-EVO-1): the screen shows the app's name, never the engine's.
//! The lookups get it from the `sub` rules of `overlay.txt` that name the engine; a text that does
//! not come from a lookup (an engine message, a profile's description, a document's name) gets it
//! from [`rename`], and what `i18n::fmt` fills in from [`renamed`]. The native format's extension
//! stays on the files and leaves the screen: a file name loses it, a message says it in words.
//! Identifiers, file formats and the ICC bytes keep the engine's name.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// The rules, as shipped (see the file's header for the format).
const RULES_TEXT: &str = include_str!("overlay.txt");

/// Rewritten strings kept at most; past this, a rewrite is still returned but no longer cached.
const MAX_CACHED: usize = 4096;

/// The engine's name, as the crates write it in their messages and profile descriptions.
const ENGINE_NAME: &str = "EvoPhoto";
/// The same word in lower case: the native format's extension and error prefix, a crate's name.
const ENGINE_WORD: &str = "evophoto";
/// How `evophoto_io`'s error for the native format starts; the sentence after it says what failed.
const ERROR_PREFIX: &str = "evophoto: ";
/// The native format's extension: the files keep it, the screen doesn't show it.
const EXTENSION: &str = ".evophoto";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// The whole string equals `from`.
    Exact,
    /// Every occurrence of `from`.
    Sub,
    /// The word `from` with the space next to it.
    Drop,
}

#[derive(Debug, PartialEq, Eq)]
struct Rule {
    kind: Kind,
    lang: String,
    from: String,
    to: String,
}

/// Parse `overlay.txt`. A malformed line is skipped (and logged), never a panic.
fn parse(text: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let (kind, columns) = match cols.first().copied() {
            Some("exact") => (Kind::Exact, 4),
            Some("sub") => (Kind::Sub, 4),
            Some("drop") => (Kind::Drop, 3),
            _ => {
                log::error!("overlay.txt line {}: unknown kind", n + 1);
                continue;
            }
        };
        let (Some(lang), Some(from)) = (cols.get(1), cols.get(2)) else {
            log::error!("overlay.txt line {}: expected kind, language, from", n + 1);
            continue;
        };
        if from.is_empty() || cols.len() != columns {
            log::error!("overlay.txt line {}: empty pattern or wrong number of columns", n + 1);
            continue;
        }
        let to = cols.get(3).copied().unwrap_or("");
        rules.push(Rule { kind, lang: (*lang).to_owned(), from: (*from).to_owned(), to: to.to_owned() });
    }
    rules
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| parse(RULES_TEXT))
}

fn applies(rule: &Rule, lang: &str) -> bool {
    rule.lang == "*" || rule.lang.eq_ignore_ascii_case(lang)
}

/// The rewrite of `s` for `lang`, or `None` when no rule touches it.
fn rewrite(rules: &[Rule], lang: &str, s: &str) -> Option<String> {
    let exact = |wanted_lang: &str| rules.iter().find(|r| r.kind == Kind::Exact && r.from == s && r.lang.eq_ignore_ascii_case(wanted_lang));
    if let Some(r) = exact(lang).or_else(|| exact("*")) {
        return Some(r.to.clone());
    }
    let mut out: Option<String> = None;
    for r in rules.iter().filter(|r| r.kind != Kind::Exact && applies(r, lang)) {
        let current = out.as_deref().unwrap_or(s);
        if !current.contains(r.from.as_str()) {
            continue;
        }
        out = Some(match r.kind {
            Kind::Drop => current.replace(&format!("{} ", r.from), "").replace(&format!(" {}", r.from), ""),
            _ => current.replace(r.from.as_str(), &r.to),
        });
    }
    out
}

/// `s` without the occurrences of `word` that `goes` picks, from the text before and after each
/// one; `None` when it picks none.
fn without(s: &str, word: &str, goes: impl Fn(&str, &str) -> bool) -> Option<String> {
    let mut out = String::new();
    let mut kept_up_to = 0;
    for (at, _) in s.match_indices(word) {
        let end = at + word.len();
        if goes(&s[..at], &s[end..]) {
            out.push_str(&s[kept_up_to..at]);
            kept_up_to = end;
        }
    }
    if kept_up_to == 0 {
        return None;
    }
    out.push_str(&s[kept_up_to..]);
    Some(out)
}

/// `s` without [`ERROR_PREFIX`], or `None` when it has none. A file name can end with the same
/// letters (`a.evophoto: no such file`): that one is for [`without_extension`].
fn without_error_prefix(s: &str) -> Option<String> {
    without(s, ERROR_PREFIX, |before, _| !before.chars().next_back().is_some_and(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-')))
}

/// `s` without [`EXTENSION`] at the end of a file name (`Saved a.evophoto`), or `None` when it has
/// none. A longer name (`a.evophoto.png`) stays; the format named on its own (` .evophoto`) is for
/// the rules of `overlay.txt`.
fn without_extension(s: &str) -> Option<String> {
    without(s, EXTENSION, |before, after| {
        // Any other character can end a name (`Affiche !.evophoto`); these stand before a format named on its own.
        let in_a_name = before.chars().next_back().is_some_and(|c| !c.is_whitespace() && !matches!(c, '(' | '[' | '*' | '`' | '"' | '\'' | '/' | '\\' | '|' | ','));
        let mut next = after.chars();
        let name_ends = match next.next() {
            None => true,
            // The end of a sentence, not a longer name.
            Some('.') => next.next().is_none_or(char::is_whitespace),
            Some(c) => !c.is_alphanumeric() && !matches!(c, '_' | '-'),
        };
        in_a_name && name_ends
    })
}

/// [`rewrite`], with [`ERROR_PREFIX`] taken off before and [`EXTENSION`] after; `None` when nothing
/// changes.
fn shown_text(rules: &[Rule], lang: &str, s: &str) -> Option<String> {
    let bare = without_error_prefix(s);
    let input = bare.as_deref().unwrap_or(s);
    let rewritten = rewrite(rules, lang, input);
    let output = rewritten.as_deref().unwrap_or(input);
    without_extension(output).or(rewritten).or(bare)
}

/// The text `make` builds for `s` in `lang`, built once and kept; `s` itself when it builds none.
fn remember<'a>(lang: &str, s: &'a str, make: impl FnOnce() -> Option<String>) -> &'a str {
    static CACHE: OnceLock<Mutex<HashMap<(String, String), &'static str>>> = OnceLock::new();
    let key = (lang.to_ascii_lowercase(), s.to_owned());
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    if let Some(&done) = cache.get(&key) {
        return done;
    }
    let Some(text) = make() else { return s };
    let done: &'static str = Box::leak(text.into_boxed_str());
    if cache.len() < MAX_CACHED {
        cache.insert(key, done);
    }
    done
}

/// `s` as the UI shows it in `lang`: unchanged (the common case, no allocation), or rewritten.
pub fn apply<'a>(lang: &str, s: &'a str) -> &'a str {
    let rules = rules();
    if !s.contains(ENGINE_WORD) && !rules.iter().any(|r| if r.kind == Kind::Exact { r.from == s } else { s.contains(r.from.as_str()) }) {
        return s;
    }
    remember(lang, s, || shown_text(rules, lang, s))
}

/// The `sub` rules that replace the engine's name, in any case.
fn name_rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| parse(RULES_TEXT).into_iter().filter(|r| r.kind == Kind::Sub && r.from.to_lowercase().contains(ENGINE_WORD)).collect())
}

fn names_the_engine(text: &str) -> bool {
    text.contains(ENGINE_NAME) || text.contains(ENGINE_WORD)
}

/// Patch EVORA-LUM-114: `text`, which does not come from a lookup (an engine message, a profile's
/// description, a document's name), with the app's name instead of the engine's. Only that
/// changes: such a text can hold what the user typed (a file name), which the other rules must not
/// rewrite. Text without the name (almost all) comes back as it is, without a lookup.
pub fn rename(text: &str) -> &str {
    if !names_the_engine(text) {
        return text;
    }
    remember("*", text, || shown_text(name_rules(), "*", text))
}

/// [`rename`] for text made on the fly (`i18n::fmt`): nothing is kept.
pub fn renamed(text: String) -> String {
    if !names_the_engine(&text) {
        return text;
    }
    shown_text(name_rules(), "*", &text).unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_rules_parse() {
        let rules = parse(RULES_TEXT);
        assert!(rules.len() >= 10, "{rules:?}");
        assert_eq!(rules.len(), RULES_TEXT.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).count());
    }

    #[test]
    fn adobe_terms_are_renamed_in_every_language() {
        assert_eq!(apply("en", "Adobe RGB (1998) compatible"), "ClayRGB (1998) compatible");
        assert_eq!(apply("de", "Adobe-RGB(1998)-kompatibel"), "ClayRGB(1998)-kompatibel");
        assert_eq!(apply("fr", "Composeur ligne par ligne Adobe"), "Composeur ligne par ligne");
        assert_eq!(apply("en", "Adobe Single-line Composer"), "Single-line Composer");
        assert_eq!(apply("en", "Adobe PDF Presets…"), "PDF Presets…");
        assert_eq!(apply("fr", "Adobe"), "Intégré");
        assert_eq!(apply("en", "Adobe"), "Built-in");
        assert_eq!(apply("hu", "Adobe egysoros szedőmotor"), "Egysoros szedőmotor");
        assert_eq!(apply("ja", "Adobe 1 行コンポーザー"), "1 行コンポーザー");
        assert_eq!(apply("pt-br", "Predefinições do Adobe PDF…"), "Predefinições do PDF…");
        assert_eq!(apply("en", "Layers"), "Layers");
    }

    /// Patch EVORA-LUM-114: whatever a lookup returns names the app, with the article it takes.
    #[test]
    fn the_lookups_name_the_app() {
        assert_eq!(apply("en", "EvoPhoto Coated CMYK (synthetic, 300% TAC, medium GCR)"), "Lumbra Coated CMYK (synthetic, 300% TAC, medium GCR)");
        assert_eq!(apply("fr", "Working: EvoPhoto Coated CMYK"), "Working: Lumbra Coated CMYK");
        assert_eq!(apply("en", "not an EvoPhoto tip"), "not a Lumbra tip");
        assert_eq!(apply("en", "An EvoPhoto droplet"), "A Lumbra droplet");
        assert_eq!(apply("en", "what can EvoPhoto open"), "what can Lumbra open");
        assert_eq!(apply("en", "Adobe RGB (1998)-compatible (EvoPhoto)"), "ClayRGB (1998)-compatible (Lumbra)");
        // The format is said in words: the files keep their extension, the screen doesn't show it.
        assert_eq!(apply("en", "save as .evophoto to keep it"), "save as a Lumbra document to keep it");
    }

    /// The About window says where the app comes from without a name, in every language.
    #[test]
    fn the_origin_is_said_without_a_name() {
        let key = "Based on EvoPhoto, open-source software (MIT OR Apache-2.0).";
        assert_eq!(super::super::tr(super::super::Lang::EN, key), "Based on open-source software (MIT OR Apache-2.0).");
        assert_eq!(apply("fr", "Basé sur EvoPhoto, logiciel libre (MIT OR Apache-2.0)."), "Basé sur un logiciel libre (MIT OR Apache-2.0).");
        for lang in super::super::Lang::all() {
            let shown = super::super::tr(lang, key);
            assert!(!shown.contains("Lumbra") && !shown.contains(ENGINE_NAME) && shown.contains("MIT OR Apache-2.0"), "{}: {shown}", lang.code());
        }
    }

    /// No catalog entry, in any language, reaches the screen with the engine's name.
    #[test]
    fn no_catalog_entry_shows_the_engine_name() {
        for lang in super::super::Lang::all() {
            let (entries, _) = super::super::catalog::parse_entries(&lang.0.text());
            for (_, source, _) in entries {
                let shown = super::super::tr(lang, &source);
                assert!(!shown.contains(ENGINE_NAME), "{}: {source:?} shows {shown:?}", lang.code());
            }
        }
    }

    /// `rename` only touches the name, for texts that do not come from a lookup.
    #[test]
    fn rename_touches_only_the_name() {
        assert_eq!(rename("File"), "File");
        assert_eq!(rename("Rec. ITU-R BT.2020 (EvoPhoto)"), "Rec. ITU-R BT.2020 (Lumbra)");
        assert_eq!(rename("a.png is not an EvoPhoto droplet"), "a.png is not a Lumbra droplet");
        assert!(std::ptr::eq(rename("EvoPhoto Defaults"), rename("EvoPhoto Defaults")), "made once");
        // A file the user named keeps its name: the other rules are not for such a text.
        assert_eq!(rename("Saved Adobe Logo.psd"), "Saved Adobe Logo.psd");
        assert_eq!(rename("Saved Adobe Logo.evophoto"), "Saved Adobe Logo");
    }

    /// What `i18n::fmt` fills in: an engine message, a file's name.
    #[test]
    fn the_values_filled_in_name_the_app() {
        assert_eq!(super::super::fmt("Saved {name}", &[("name", "poster.evophoto")]), "Saved poster");
        assert_eq!(super::super::fmt("Failed: {error}", &[("error", "not an EvoPhoto tip")]), "Failed: not a Lumbra tip");
        assert_eq!(renamed("Saved a.png".to_owned()), "Saved a.png");
    }

    /// The native format's errors lose their prefix, even after a file name with the same letters.
    #[test]
    fn the_error_prefix_goes() {
        assert_eq!(rename("evophoto: corrupt .evophoto bundle: x"), "corrupt Lumbra document: x");
        assert_eq!(rename("a.png: evophoto: corrupt .evophoto bundle: x"), "a.png: corrupt Lumbra document: x");
        assert_eq!(rename("a.evophoto: evophoto: corrupt .evophoto bundle: x"), "a: corrupt Lumbra document: x");
        assert_eq!(apply("en", "Couldn't open a.evophoto: evophoto: limit exceeded: y"), "Couldn't open a: limit exceeded: y");
    }

    /// A file name loses the extension at its end, and only there.
    #[test]
    fn file_names_lose_the_extension() {
        assert_eq!(rename("poster.evophoto"), "poster");
        assert_eq!(rename("poster (1).evophoto copy"), "poster (1) copy");
        assert_eq!(rename("“poster.evophoto”"), "“poster”");
        assert_eq!(rename("a.evophoto: no such file"), "a: no such file");
        assert_eq!(rename("Couldn't open a.evophoto."), "Couldn't open a.");
        assert_eq!(rename("a.evophoto.png"), "a.evophoto.png");
        assert_eq!(rename("a.evophotos"), "a.evophotos");
        assert_eq!(rename("Affiche !.evophoto"), "Affiche !");
        assert_eq!(rename("(*.evophoto)"), "(*.evophoto)");
    }

    /// The engine's messages that name the native format or a crate, as the crates write them.
    #[test]
    fn the_format_is_said_in_words() {
        assert_eq!(
            rename("XCF is GIMP's own format and EvoPhoto only opens it; save as .evophoto to keep the layers, or export a flat image"),
            "XCF is GIMP's own format and Lumbra only opens it; save as a Lumbra document to keep the layers, or export a flat image"
        );
        assert_eq!(rename("Affinity export is not implemented; save a new PSD, PNG or .evophoto copy"), "Affinity export is not implemented; save a new PSD, PNG or Lumbra copy");
        assert_eq!(
            rename("Krita files are import-only; save as .ora to open the layers in Krita, or .evophoto to keep everything"),
            "Krita files are import-only; save as .ora to open the layers in Krita, or a Lumbra document to keep everything"
        );
        assert_eq!(rename("`a` is NaN or infinite, which an .evophoto manifest cannot hold"), "`a` is NaN or infinite, which a Lumbra manifest cannot hold");
        assert_eq!(rename("raw files are not flat images; decode them with evophoto-raw"), "raw files are not flat images; decode them with Lumbra");
    }
}
