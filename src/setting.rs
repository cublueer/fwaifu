use std::io::{self, Write};

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
    MenuItem { label: "Terminal display (chafa)",      path: &["term"],              kind: Kind::Bool },
    MenuItem { label: "Chafa width",                    path: &["term_width"],        kind: Kind::Int },
    MenuItem { label: "Proxy URL",                      path: &["proxy"],             kind: Kind::Str },
    // ── crop ──
    MenuItem { label: "Image cropping",                 path: &["crop"],              kind: Kind::Bool },
    MenuItem { label: "Crop width (px)",                path: &["crop_width"],        kind: Kind::Int },
    MenuItem { label: "Crop height (px)",               path: &["crop_height"],       kind: Kind::Int },
    // ── display ──
    MenuItem { label: "Logo width",                     path: &["logo_width"],        kind: Kind::Int },
    MenuItem { label: "Watch interval (s)",             path: &["watch_interval"],    kind: Kind::Int },
    // ── [download] ──
    MenuItem { label: "Download batch size",            path: &["download", "batch_size"], kind: Kind::Int },
    // ── [cache] ──
    MenuItem { label: "Cache max limit",                path: &["cache", "max_limit"],    kind: Kind::Int },
    MenuItem { label: "Cache min trigger",              path: &["cache", "min_trigger"],  kind: Kind::Int },
    MenuItem { label: "Cache max used",                 path: &["cache", "max_used"],     kind: Kind::Int },
    MenuItem { label: "Cache auto-clean",               path: &["cache", "clean_cache"],  kind: Kind::Bool },
    // ── save ──
    MenuItem { label: "SFW save path",                  path: &["save_path_sfw"],     kind: Kind::Str },
    MenuItem { label: "NSFW save path",                 path: &["save_path_nsfw"],    kind: Kind::Str },
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
        let input = read_line("\nEnter number to modify, s to save, q to quit: ").unwrap_or_default();

        match input.trim() {
            "q" | "Q" => {
                println!("Exited without saving.");
                return Ok(());
            }
            "s" | "S" => {
                let parent = config_path.parent().unwrap();
                std::fs::create_dir_all(parent)?;
                std::fs::write(&config_path, doc.to_string())?;
                println!("\nConfig saved to {}", config_path.display());
                return Ok(());
            }
            num_str => {
                match num_str.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= ITEMS.len() => {
                        if let Err(e) = edit_item(&ITEMS[n - 1], &mut doc) {
                            println!("Error: {}", e);
                        }
                    }
                    _ => println!("Invalid. Enter 1-{}, s, or q.", ITEMS.len()),
                }
            }
        }
    }
}

fn print_menu(doc: &toml_edit::DocumentMut) {
    println!("\n  fwaifu Configuration Editor");
    println!("  ────────────────────────────\n");

    for (i, item) in ITEMS.iter().enumerate() {
        let val = display_value(doc, item);
        println!("  {:2}. {:<30} = {}", i + 1, item.label, val);
    }
}

fn display_value(doc: &toml_edit::DocumentMut, item: &MenuItem) -> String {
    let target = resolve(doc, item.path);
    match item.kind {
        Kind::Bool => match target.and_then(|t| t.as_bool()) {
            Some(v) => v.to_string(),
            None => "(not set)".to_string(),
        },
        Kind::Int => match target.and_then(|t| t.as_integer()) {
            Some(v) => v.to_string(),
            None => "(not set)".to_string(),
        },
        Kind::Str => match target.and_then(|t| t.as_str()) {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => "(not set)".to_string(),
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

fn edit_item(item: &MenuItem, doc: &mut toml_edit::DocumentMut) -> Result<(), String> {
    let current = display_value(doc, item);
    println!("\n  {}  (current: {})", item.label, current);

    match item.kind {
        Kind::Bool => {
            let cur = resolve(doc, item.path).and_then(|t| t.as_bool()).unwrap_or(false);
            let new_val = !cur;
            let display = if new_val { "true" } else { "false" };
            let input = read_line(&format!("  Change to {}? [y/N] ", display)).unwrap_or_default();
            if input.trim().eq_ignore_ascii_case("y") {
                set_value(doc, item.path, toml_edit::value(new_val));
                println!("  {} = {}", item.label, display);
            } else {
                println!("  Unchanged.");
            }
        }
        Kind::Int => {
            let input = read_line("  New value (empty to keep): ").unwrap_or_default();
            let trimmed = input.trim();
            if trimmed.is_empty() {
                println!("  Unchanged.");
                return Ok(());
            }
            match trimmed.parse::<i64>() {
                Ok(v) if v >= 0 => {
                    set_value(doc, item.path, toml_edit::value(v));
                    println!("  {} = {}", item.label, v);
                }
                _ => return Err("Invalid number (must be >= 0)".to_string()),
            }
        }
        Kind::Str => {
            let input = read_line("  New value (empty to clear): ").unwrap_or_default();
            let trimmed = input.trim();
            if trimmed.is_empty() {
                remove_key(doc, item.path);
                println!("  {} cleared.", item.label);
            } else {
                set_value(doc, item.path, toml_edit::value(trimmed));
                println!("  {} = {}", item.label, trimmed);
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
