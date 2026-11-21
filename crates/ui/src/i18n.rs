//! The program's own texts, in Fluent (`locales/<lang>.ftl`).
//!
//! Every text a person reads goes through [`I18n::tr`]; widgets receive
//! texts already translated. The `locales_have_the_same_messages` test keeps
//! the languages in step. Texts that come with plugins and theme packs are
//! translated in their own files, as [`psito3d_scene::LocalizedText`].

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};

/// The languages the program speaks: the code and the name in itself.
pub const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("pt", "Português")];

fn source(lang: &str) -> Option<&'static str> {
    match lang {
        "en" => Some(include_str!("../locales/en.ftl")),
        "pt" => Some(include_str!("../locales/pt.ftl")),
        _ => None,
    }
}

fn bundle(lang: &str, text: &'static str) -> FluentBundle<FluentResource> {
    let id = lang.parse().expect("the language codes are valid");
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // No Unicode isolation marks: egui would draw them as boxes.
    bundle.set_use_isolating(false);
    let resource = FluentResource::try_new(text.to_owned()).expect("the shipped .ftl files parse");
    bundle.add_resource(resource).expect("no message is defined twice");
    bundle
}

/// The translations for one language, with English behind them for any
/// message the language is missing.
pub struct I18n {
    lang: &'static str,
    bundle: FluentBundle<FluentResource>,
    fallback: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n").field("lang", &self.lang).finish_non_exhaustive()
    }
}

impl I18n {
    /// The translations for `lang` (`pt`, `pt-BR`, `pt_BR.UTF-8`); an
    /// unknown language gets English.
    pub fn new(lang: &str) -> Self {
        let base = lang.split(['-', '_', '.']).next().unwrap_or("en").to_ascii_lowercase();
        let lang = LANGUAGES.iter().map(|(code, _)| *code).find(|c| *c == base).unwrap_or("en");
        let en = source("en").expect("English exists");
        Self { lang, bundle: bundle(lang, source(lang).unwrap_or(en)), fallback: bundle("en", en) }
    }

    /// The language in use, such as `pt`.
    pub fn lang(&self) -> &'static str {
        self.lang
    }

    /// The text of message `id`.
    pub fn tr(&self, id: &str) -> String {
        self.tr_args(id, &[])
    }

    /// The text of message `id`, with `{ $name }` placeholders filled in.
    pub fn tr_args(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.clone());
        }
        for bundle in [&self.bundle, &self.fallback] {
            if let Some(pattern) = bundle.get_message(id).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                if !errors.is_empty() {
                    log::warn!("message {id}: {errors:?}");
                }
                return text.into_owned();
            }
        }
        log::warn!("missing message {id}");
        id.to_owned()
    }
}

/// The message ids of a language file, for the tests.
pub fn message_ids(lang: &str) -> Vec<String> {
    let text = source(lang).unwrap_or_default();
    let resource = FluentResource::try_new(text.to_owned()).expect("parses");
    let mut ids: Vec<String> = resource
        .entries()
        .filter_map(|e| match e {
            fluent_syntax::ast::Entry::Message(m) => Some(m.id.name.to_owned()),
            _ => None,
        })
        .collect();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_have_the_same_messages() {
        let en = message_ids("en");
        assert!(!en.is_empty());
        for (lang, _) in LANGUAGES {
            assert_eq!(message_ids(lang), en, "{lang} differs from en");
        }
    }

    #[test]
    fn picks_the_language_and_falls_back() {
        assert_eq!(I18n::new("pt_BR.UTF-8").lang(), "pt");
        assert_eq!(I18n::new("fr").lang(), "en");
        let pt = I18n::new("pt");
        assert_ne!(pt.tr("surprise-me"), I18n::new("en").tr("surprise-me"));
        assert_eq!(pt.tr("no-such-message"), "no-such-message");
    }

    #[test]
    fn fills_placeholders() {
        let en = I18n::new("en");
        let text = en.tr_args("status-samples", &[("done", 3.into()), ("total", 256.into())]);
        assert!(text.contains('3') && text.contains("256"), "{text}");
    }
}
