pub mod translator;

pub use translator::Translator;

pub fn detect_system_locale() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "(Get-Culture).TwoLetterISOLanguageName"])
            .output()
        {
            let lang = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            if !lang.is_empty() {
                return lang_to_locale(&lang);
            }
        }
        "en".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        detect_linux_locale()
    }
}

fn lang_to_locale(lang: &str) -> String {
    let lang = lang.trim().to_lowercase();
    if lang.starts_with("es") {
        "es".to_string()
    } else if lang.starts_with("pt") {
        "pt".to_string()
    } else if lang.starts_with("fr") {
        "fr".to_string()
    } else if lang.starts_with("de") {
        "de".to_string()
    } else if lang.starts_with("it") {
        "it".to_string()
    } else if lang.starts_with("ja") {
        "ja".to_string()
    } else if lang.starts_with("zh") {
        "zh".to_string()
    } else if lang.starts_with("ru") {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_linux_locale() -> String {
    let from_env = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .ok();
    if let Some(lang) = from_env {
        if !lang.is_empty() {
            return lang_to_locale(&lang);
        }
    }

    if let Ok(content) = std::fs::read_to_string("/etc/default/locale") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("LANG=") {
                let lang = val.trim_matches('"');
                if !lang.is_empty() {
                    return lang_to_locale(lang);
                }
            }
        }
    }

    if let Ok(output) = std::process::Command::new("locale").output() {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if let Some(val) = line.strip_prefix("LANG=") {
                let lang = val.trim_matches('"');
                if !lang.is_empty() {
                    return lang_to_locale(lang);
                }
            }
        }
    }

    "en".to_string()
}

#[macro_export]
macro_rules! tr {
    ($t:expr, $key:literal) => {
        $t.get($key)
    };
    ($t:expr, $key:literal, $($arg:expr),+) => {
        $t.get_fmt($key, &[$(format!("{}", $arg)),+])
    };
}
