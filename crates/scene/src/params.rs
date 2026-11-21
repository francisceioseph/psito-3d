//! Parameters that a terrain generator (built in or from a plugin) exposes.
//!
//! A generator *describes* its parameters with [`ParamSpec`]s, and the editor
//! builds the controls from them, so a plugin author never writes UI code.
//! The values chosen live in the scene as [`ParamValues`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A text in several languages, keyed by language code (`en`, `pt`).
///
/// In a manifest it can be written as a plain string, which counts as
/// English, or as a table: `label = { en = "Height", pt = "Altura" }`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "LocalizedRepr", into = "BTreeMap<String, String>")]
pub struct LocalizedText(BTreeMap<String, String>);

#[derive(Deserialize)]
#[serde(untagged)]
enum LocalizedRepr {
    Plain(String),
    Map(BTreeMap<String, String>),
}

impl From<LocalizedRepr> for LocalizedText {
    fn from(repr: LocalizedRepr) -> Self {
        match repr {
            LocalizedRepr::Plain(text) => Self::new(text),
            LocalizedRepr::Map(map) => Self(map),
        }
    }
}

impl From<LocalizedText> for BTreeMap<String, String> {
    fn from(text: LocalizedText) -> Self {
        text.0
    }
}

impl LocalizedText {
    /// A text with only the English version.
    pub fn new(english: impl Into<String>) -> Self {
        Self(BTreeMap::from([("en".to_owned(), english.into())]))
    }

    /// Adds the version in `lang`.
    pub fn with(mut self, lang: &str, text: impl Into<String>) -> Self {
        self.0.insert(lang.to_owned(), text.into());
        self
    }

    /// The text in `lang` (`pt` or `pt-BR`), falling back to English and then
    /// to any language there is. Empty if there is no text at all.
    pub fn get(&self, lang: &str) -> &str {
        let base = lang.split(['-', '_']).next().unwrap_or(lang);
        [lang, base, "en"]
            .iter()
            .find_map(|l| self.0.get(*l))
            .or_else(|| self.0.values().next())
            .map_or("", String::as_str)
    }
}

/// One option of a [`ParamKind::Choice`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOption {
    /// The value stored in the scene.
    pub id: String,
    /// What the person reads.
    pub label: LocalizedText,
}

/// The kind of control and the range of a parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ParamKind {
    /// A number between `min` and `max`.
    Slider {
        /// The smallest value.
        min: f32,
        /// The largest value.
        max: f32,
        /// The value of a new scene.
        default: f32,
    },
    /// On or off.
    Toggle {
        /// The value of a new scene.
        default: bool,
    },
    /// One of a few named options.
    Choice {
        /// The options, in the order shown.
        options: Vec<ChoiceOption>,
        /// The id of the option of a new scene.
        default: String,
    },
}

/// The description of a parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamSpec {
    /// The key in [`ParamValues`] and in the Lua table the plugin receives.
    pub id: String,
    /// The label of the control.
    pub label: LocalizedText,
    /// The control and its range.
    #[serde(flatten)]
    pub kind: ParamKind,
}

/// The value of one parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// The value of a slider.
    Number(f32),
    /// The value of a toggle.
    Bool(bool),
    /// The id of the chosen option.
    Choice(String),
}

/// The values of a generator's parameters, by [`ParamSpec::id`].
pub type ParamValues = BTreeMap<String, ParamValue>;

impl ParamSpec {
    /// A slider.
    pub fn slider(id: &str, label: LocalizedText, min: f32, max: f32, default: f32) -> Self {
        Self { id: id.to_owned(), label, kind: ParamKind::Slider { min, max, default } }
    }

    /// A toggle.
    pub fn toggle(id: &str, label: LocalizedText, default: bool) -> Self {
        Self { id: id.to_owned(), label, kind: ParamKind::Toggle { default } }
    }

    /// The value of a new scene.
    pub fn default_value(&self) -> ParamValue {
        match &self.kind {
            ParamKind::Slider { default, .. } => ParamValue::Number(*default),
            ParamKind::Toggle { default } => ParamValue::Bool(*default),
            ParamKind::Choice { default, .. } => ParamValue::Choice(default.clone()),
        }
    }

    /// `value` made valid for this parameter: a number is clamped to the
    /// range, and a missing value, a value of the wrong type or an unknown
    /// option becomes the default. Scene files and plugins can be edited by
    /// hand, so nothing read from them is trusted as is.
    pub fn sanitize(&self, value: Option<&ParamValue>) -> ParamValue {
        match (&self.kind, value) {
            (ParamKind::Slider { min, max, .. }, Some(ParamValue::Number(n))) if n.is_finite() => {
                ParamValue::Number(n.clamp(*min, *max))
            }
            (ParamKind::Toggle { .. }, Some(ParamValue::Bool(b))) => ParamValue::Bool(*b),
            (ParamKind::Choice { options, .. }, Some(ParamValue::Choice(id)))
                if options.iter().any(|o| &o.id == id) =>
            {
                ParamValue::Choice(id.clone())
            }
            _ => self.default_value(),
        }
    }

    /// Checks that the description itself makes sense.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || !self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!("parameter id {:?} must be letters, digits or _", self.id));
        }
        match &self.kind {
            ParamKind::Slider { min, max, default } => {
                if !(min.is_finite() && max.is_finite() && min < max) {
                    return Err(format!("parameter {}: min must be below max", self.id));
                }
                if !(min..=max).contains(&default) {
                    return Err(format!("parameter {}: default outside the range", self.id));
                }
            }
            ParamKind::Toggle { .. } => {}
            ParamKind::Choice { options, default } => {
                if !options.iter().any(|o| &o.id == default) {
                    return Err(format!("parameter {}: default is not an option", self.id));
                }
            }
        }
        Ok(())
    }
}

/// Every parameter of `specs` with a valid value, taken from `values` when
/// it has one. Values for parameters that no longer exist are dropped.
pub fn resolve(specs: &[ParamSpec], values: &ParamValues) -> ParamValues {
    specs.iter().map(|s| (s.id.clone(), s.sanitize(values.get(&s.id)))).collect()
}

impl ParamValue {
    /// The number, or `fallback` if this is not a number.
    pub fn as_f32(&self, fallback: f32) -> f32 {
        match self {
            Self::Number(n) => *n,
            _ => fallback,
        }
    }

    /// The flag, or `fallback` if this is not a flag.
    pub fn as_bool(&self, fallback: bool) -> bool {
        match self {
            Self::Bool(b) => *b,
            _ => fallback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn height() -> ParamSpec {
        ParamSpec::slider("height", LocalizedText::new("Height").with("pt", "Altura"), 0.0, 1.0, 0.6)
    }

    #[test]
    fn localized_text_falls_back() {
        let t = LocalizedText::new("Height").with("pt", "Altura");
        assert_eq!(t.get("pt"), "Altura");
        assert_eq!(t.get("pt-BR"), "Altura");
        assert_eq!(t.get("fr"), "Height");
        assert_eq!(LocalizedText::default().get("en"), "");
    }

    #[test]
    fn localized_text_reads_a_plain_string() {
        let t: LocalizedText = serde_json::from_str("\"Height\"").unwrap();
        assert_eq!(t.get("pt"), "Height");
    }

    #[test]
    fn sanitize_clamps_and_replaces() {
        let s = height();
        assert_eq!(s.sanitize(Some(&ParamValue::Number(3.0))), ParamValue::Number(1.0));
        assert_eq!(s.sanitize(Some(&ParamValue::Number(f32::NAN))), ParamValue::Number(0.6));
        assert_eq!(s.sanitize(Some(&ParamValue::Bool(true))), ParamValue::Number(0.6));
        assert_eq!(s.sanitize(None), ParamValue::Number(0.6));
    }

    #[test]
    fn choice_rejects_unknown_options() {
        let s = ParamSpec {
            id: "shape".into(),
            label: LocalizedText::new("Shape"),
            kind: ParamKind::Choice {
                options: vec![
                    ChoiceOption { id: "round".into(), label: LocalizedText::new("Round") },
                    ChoiceOption { id: "long".into(), label: LocalizedText::new("Long") },
                ],
                default: "round".into(),
            },
        };
        assert!(s.validate().is_ok());
        let long = ParamValue::Choice("long".into());
        assert_eq!(s.sanitize(Some(&long)), long);
        assert_eq!(s.sanitize(Some(&ParamValue::Choice("x".into()))), ParamValue::Choice("round".into()));
    }

    #[test]
    fn validate_catches_bad_specs() {
        let mut s = height();
        s.kind = ParamKind::Slider { min: 1.0, max: 0.0, default: 0.5 };
        assert!(s.validate().is_err());
        let mut s = height();
        s.id = "has space".into();
        assert!(s.validate().is_err());
    }

    #[test]
    fn resolve_fills_and_drops() {
        let mut values = ParamValues::new();
        values.insert("gone".into(), ParamValue::Bool(true));
        let resolved = resolve(&[height()], &values);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved["height"], ParamValue::Number(0.6));
    }
}
