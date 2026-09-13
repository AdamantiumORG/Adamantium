pub fn executable_name(name: &str, windows: bool) -> String {
    if windows {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}
