use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/target/man"]
#[include = "*.txt"]
struct ManPages;

pub fn get(command: String) -> Option<String> {
    return ManPages::get(
        &(String::from("cp437-") + command.trim_start_matches("cp437-") + ".txt"),
    )
    .map(|file| return String::from_utf8(file.data.into_owned()).unwrap());
}

pub fn print(command: String) {
    eprintln!("{}", get(command).unwrap());
}
