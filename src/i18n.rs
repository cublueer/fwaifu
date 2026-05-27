use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    EnUS,
    ZhCN,
}

/// Detect the user's preferred language from the LANG environment variable.
///
/// Parses locale strings like `zh_CN.UTF-8`, `en_US.utf8`, `zh_CN`, etc.
/// Extracts the locale part before the encoding suffix, then matches on the
/// language+territory prefix.
pub fn detect_lang() -> Lang {
    let lang = std::env::var("LANG").unwrap_or_default();
    // "zh_CN.UTF-8" → "zh_CN"
    let locale = lang.split('.').next().unwrap_or(&lang);

    if locale.starts_with("zh_") {
        Lang::ZhCN
    } else {
        Lang::EnUS
    }
}

/// Parse a TOML translations table into a flat key-value map.
///
/// Sections are flattened: `[msg]` with key `hello` becomes `"msg.hello"`.
/// Top-level keys are kept as-is.
fn flatten_toml(raw: &str) -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    let Ok(table) = raw.parse::<toml::Table>() else {
        return map;
    };

    for (section_key, value) in &table {
        if let Some(sub) = value.as_table() {
            for (key, val) in sub {
                if let Some(s) = val.as_str() {
                    let full_key = format!("{section_key}.{key}");
                    map.insert(
                        Box::leak(full_key.into_boxed_str()),
                        Box::leak(s.to_string().into_boxed_str()),
                    );
                }
            }
        } else if let Some(s) = value.as_str() {
            map.insert(
                Box::leak(section_key.clone().into_boxed_str()),
                Box::leak(s.to_string().into_boxed_str()),
            );
        }
    }

    map
}

/// All translations, loaded once on first access.
/// Both language files are embedded at compile time via `include_str!`.
static STRINGS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let raw = match detect_lang() {
        Lang::ZhCN => include_str!("../i18n/zh_CN.toml"),
        Lang::EnUS => include_str!("../i18n/en_US.toml"),
    };
    flatten_toml(raw)
});

/// Translate a plain string by key.
///
/// Returns the key itself as fallback if not found.
///
/// # Example
///
/// ```ignore
/// println!("{}", i18n::t("msg.logged_in"));
/// ```
pub fn t(key: &str) -> &str {
    STRINGS.get(key).copied().unwrap_or(key)
}

/// Translate a string with format arguments.
///
/// Placeholders `{0}`, `{1}`, etc. in the translation are replaced
/// with the corresponding argument.
///
/// # Example
///
/// ```ignore
/// eprintln!("{}", i18n::tf("error.login_failed", &[&e.to_string()]));
/// ```
pub fn tf(key: &str, args: &[&str]) -> String {
    let tmpl = STRINGS.get(key).copied().unwrap_or(key);
    let mut result = tmpl.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{i}}}"), arg);
    }
    result
}
