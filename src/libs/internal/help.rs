use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/man"]
#[include = "*.txt"]
struct ManPages;

#[must_use]
pub fn get(command: &str) -> Option<String> {
    return ManPages::get(&(String::from("cp437-") + command.trim_start_matches("cp437-") + ".txt"))
        .map(|file| return String::from_utf8(file.data.into_owned()).expect("Man pages are valid UTF-8"));
}

pub fn print(command: &str) -> Result<(), String> {
    if let Some(text) = get(command) {
        eprintln!("{text}");
        return Ok(());
    }

    return Err(format!("Help text for command `{command}` not found"));
}
