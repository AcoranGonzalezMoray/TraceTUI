use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Translator {
    strings: HashMap<String, String>,
    pub locale: String,
}

impl Translator {
    pub fn new(locale: &str) -> Self {
        let (data, locale) = match locale {
            "es" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("es.json")),
                "es",
            ),
            "pt" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("pt.json")),
                "pt",
            ),
            "fr" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("fr.json")),
                "fr",
            ),
            "de" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("de.json")),
                "de",
            ),
            "it" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("it.json")),
                "it",
            ),
            "ja" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("ja.json")),
                "ja",
            ),
            "zh" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("zh.json")),
                "zh",
            ),
            "ru" => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("ru.json")),
                "ru",
            ),
            _ => (
                serde_json::from_str::<HashMap<String, String>>(include_str!("en.json")),
                "en",
            ),
        };
        Self {
            strings: data.unwrap_or_default(),
            locale: locale.to_string(),
        }
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    pub fn get_fmt(&self, key: &str, args: &[String]) -> String {
        let template = self.get(key);
        let mut result =
            String::with_capacity(template.len() + args.iter().map(|a| a.len()).sum::<usize>());
        let mut rest = template;
        let mut arg_index = 0;
        while let Some(pos) = rest.find("{}") {
            result.push_str(&rest[..pos]);
            if let Some(arg) = args.get(arg_index) {
                result.push_str(arg);
                arg_index += 1;
            }
            rest = &rest[pos + 2..];
        }
        result.push_str(rest);
        result
    }

    pub fn available_locales() -> Vec<(&'static str, &'static str)> {
        vec![
            ("en", "English"),
            ("es", "Español"),
            ("pt", "Português"),
            ("fr", "Français"),
            ("de", "Deutsch"),
            ("it", "Italiano"),
            ("ja", "日本語"),
            ("zh", "中文"),
            ("ru", "Русский"),
        ]
    }
}
