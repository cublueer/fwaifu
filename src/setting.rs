use std::io::{self, Write};

use crate::i18n;

#[derive(Clone, Copy)]
enum Kind {
    Bool,
    Int,
    Str,
}

struct MenuItem {
    label: &'static str,
    path: &'static [&'static str],
    kind: Kind,
}

const ITEMS: &[MenuItem] = &[
    // ── display ──
    MenuItem { label: "setting.menu_0",  path: &["term"],              kind: Kind::Bool },
    MenuItem { label: "setting.menu_1",  path: &["term_width"],        kind: Kind::Int },
    MenuItem { label: "setting.menu_2",  path: &["proxy"],             kind: Kind::Str },
    // ── crop ──
    MenuItem { label: "setting.menu_3",  path: &["crop"],              kind: Kind::Bool },
    MenuItem { label: "setting.menu_4",  path: &["crop_width"],        kind: Kind::Int },
    MenuItem { label: "setting.menu_5",  path: &["crop_height"],       kind: Kind::Int },
    // ── display ──
    MenuItem { label: "setting.menu_6",  path: &["logo_width"],        kind: Kind::Int },
    MenuItem { label: "setting.menu_7",  path: &["watch_interval"],    kind: Kind::Int },
    // ── [download] ──
    MenuItem { label: "setting.menu_8",  path: &["download", "batch_size"], kind: Kind::Int },
    // ── [cache] ──
    MenuItem { label: "setting.menu_9",  path: &["cache", "max_limit"],    kind: Kind::Int },
    MenuItem { label: "setting.menu_10", path: &["cache", "min_trigger"],  kind: Kind::Int },
    MenuItem { label: "setting.menu_11", path: &["cache", "max_used"],     kind: Kind::Int },
    MenuItem { label: "setting.menu_12", path: &["cache", "clean_cache"],  kind: Kind::Bool },
    // ── save ──
    MenuItem { label: "setting.menu_13", path: &["save_path_sfw"],     kind: Kind::Str },
    MenuItem { label: "setting.menu_14", path: &["save_path_nsfw"],    kind: Kind::Str },
];

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("fwaifu")
        .join("config.toml");

    let mut doc = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        content.parse::<toml_edit::DocumentMut>()?
    } else {
        let parent = config_path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        toml_edit::DocumentMut::new()
    };

    ensure_table(&mut doc, "download");
    ensure_table(&mut doc, "cache");

    loop {
        print_menu(&doc);
        let prompt = format!("\n{}", i18n::t("setting.prompt"));
        let input = read_line(&prompt).unwrap_or_default();

        match input.trim() {
            "q" | "Q" => {
                println!("{}", i18n::t("setting.exited"));
                return Ok(());
            }
            "s" | "S" => {
                let parent = config_path.parent().unwrap();
                std::fs::create_dir_all(parent)?;
                std::fs::write(&config_path, doc.to_string())?;
                println!("\n{}", i18n::tf("setting.saved", &[&config_path.display().to_string()]));
                return Ok(());
            }
            "r" | "R" => {
                let prompt = i18n::t("setting.restore_confirm");
                let answer = read_line(prompt).unwrap_or_default();
                if answer.trim().eq_ignore_ascii_case("y") {
                    for item in ITEMS {
                        remove_key(&mut doc, item.path);
                    }
                    cleanup_empty_table(&mut doc, "download");
                    cleanup_empty_table(&mut doc, "cache");
                    println!("{}", i18n::t("setting.restored"));
                } else {
                    println!("{}", i18n::t("setting.restore_cancelled"));
                }
            }
            num_str => {
                match num_str.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= ITEMS.len() => {
                        if let Err(e) = edit_item(&ITEMS[n - 1], &mut doc) {
                            println!("{}", i18n::tf("setting.error", &[&e]));
                        }
                    }
                    _ => println!("{}", i18n::tf("setting.invalid", &[&ITEMS.len().to_string()])),
                }
            }
        }
    }
}

fn print_menu(doc: &toml_edit::DocumentMut) {
    let title = i18n::t("setting.title");
    println!("\n  {title}");
    println!("  {}", "─".repeat(display_width(title)));

    for (i, item) in ITEMS.iter().enumerate() {
        let val = display_value(doc, item);
        let label = i18n::t(item.label);
        let pw = display_width(label);
        let pad = 30usize.saturating_sub(pw);
        println!("  {:2}. {}{} = {}", i + 1, label, " ".repeat(pad.max(1)), val);
    }
}

/// Count terminal display columns: CJK/punctuation = 2, ASCII = 1.
fn display_width(s: &str) -> usize {
    s.chars().map(|c| {
        if ('\u{1100}'..'\u{115F}').contains(&c)     // Hangul Jamo
            || ('\u{2329}'..='\u{232A}').contains(&c)  // angle brackets
            || ('\u{2E80}'..='\u{303E}').contains(&c)  // CJK radicals + symbols
            || ('\u{3040}'..='\u{A4CF}').contains(&c)  // Hiragana, Katakana, Bopomofo, Hangul, CJK Unified, Yi
            || ('\u{AC00}'..='\u{D7A3}').contains(&c)  // Hangul Syllables
            || ('\u{F900}'..='\u{FAFF}').contains(&c)  // CJK Compat
            || ('\u{FE10}'..='\u{FE19}').contains(&c)  // vertical forms
            || ('\u{FE30}'..='\u{FE6F}').contains(&c)  // CJK Compat Forms
            || ('\u{FF00}'..='\u{FF60}').contains(&c)  // Fullwidth Forms
            || ('\u{FFE0}'..='\u{FFE6}').contains(&c)  // Fullwidth Signs
            || ('\u{20000}'..='\u{2FFFF}').contains(&c) // CJK Extension B+
            || ('\u{30000}'..='\u{3FFFF}').contains(&c) // CJK Extension G+
        { 2 } else { 1 }
    }).sum()
}

fn display_value(doc: &toml_edit::DocumentMut, item: &MenuItem) -> String {
    let target = resolve(doc, item.path);
    match item.kind {
        Kind::Bool => match target.and_then(|t| t.as_bool()) {
            Some(v) => v.to_string(),
            None => i18n::t("setting.not_set").to_string(),
        },
        Kind::Int => match target.and_then(|t| t.as_integer()) {
            Some(v) => v.to_string(),
            None => i18n::t("setting.not_set").to_string(),
        },
        Kind::Str => match target.and_then(|t| t.as_str()) {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => i18n::t("setting.not_set").to_string(),
        },
    }
}

fn resolve<'a>(doc: &'a toml_edit::DocumentMut, path: &[&str]) -> Option<&'a toml_edit::Item> {
    if path.len() == 1 {
        return doc.get(path[0]);
    }
    doc.get(path[0]).and_then(|t| t.get(path[1]))
}

fn ensure_table(doc: &mut toml_edit::DocumentMut, name: &str) {
    if !doc.contains_table(name) {
        doc[name] = toml_edit::Item::Table(toml_edit::Table::new());
    }
}

fn cleanup_empty_table(doc: &mut toml_edit::DocumentMut, name: &str) {
    if let Some(table) = doc.get(name).and_then(|t| t.as_table()) {
        if table.is_empty() {
            doc.remove(name);
        }
    }
}

fn edit_item(item: &MenuItem, doc: &mut toml_edit::DocumentMut) -> Result<(), String> {
    let label = i18n::t(item.label);
    let current = display_value(doc, item);
    println!("\n  {}  ({})", label, i18n::tf("setting.current", &[&current]));

    match item.kind {
        Kind::Bool => {
            let cur = resolve(doc, item.path).and_then(|t| t.as_bool()).unwrap_or(false);
            let new_val = !cur;
            let display = if new_val { "true" } else { "false" };
            let prompt = i18n::tf("setting.change", &[display]);
            let input = read_line(&prompt).unwrap_or_default();
            if input.trim().eq_ignore_ascii_case("y") {
                set_value(doc, item.path, toml_edit::value(new_val));
                println!("  {} = {}", label, display);
            } else {
                println!("  {}", i18n::t("setting.unchanged"));
            }
        }
        Kind::Int => {
            let input = read_line(i18n::t("setting.int_prompt")).unwrap_or_default();
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("  {}", i18n::t("setting.unchanged"));
                return Ok(());
            }
            match trimmed.parse::<i64>() {
                Ok(v) if v >= 0 => {
                    set_value(doc, item.path, toml_edit::value(v));
                    println!("  {} = {}", label, v);
                }
                _ => return Err(i18n::t("setting.int_invalid").to_string()),
            }
        }
        Kind::Str => {
            let input = read_line(i18n::t("setting.str_prompt")).unwrap_or_default();
            let trimmed = input.trim();
            if trimmed.is_empty() {
                remove_key(doc, item.path);
                println!("  {}", i18n::tf("setting.cleared", &[label]));
            } else {
                set_value(doc, item.path, toml_edit::value(trimmed));
                println!("  {} = {}", label, trimmed);
            }
        }
    }
    Ok(())
}

fn set_value(doc: &mut toml_edit::DocumentMut, path: &[&str], item: toml_edit::Item) {
    match path.len() {
        1 => { doc[path[0]] = item; }
        2 => {
            if !doc.contains_table(path[0]) {
                doc[path[0]] = toml_edit::Item::Table(toml_edit::Table::new());
            }
            doc[path[0]][path[1]] = item;
        }
        _ => {}
    }
}

fn remove_key(doc: &mut toml_edit::DocumentMut, path: &[&str]) {
    match path.len() {
        1 => { doc.remove(path[0]); }
        2 => {
            if let Some(table) = doc.get_mut(path[0]).and_then(|t| t.as_table_mut()) {
                table.remove(path[1]);
            }
        }
        _ => {}
    }
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s)
}
