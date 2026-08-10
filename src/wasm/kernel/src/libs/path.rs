pub fn split_filename(path: String) -> Option<(String, String)> {
    // /a/b/c -> ("/a/b", "c")
    // /a/b/c/ -> ("/a/b", "c")
    // / -> None
    // /a -> ("/", "a")

    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }

    let mut parts = path.split('/').collect::<Vec<_>>();
    let filename = parts.pop().unwrap().to_string();

    if parts.is_empty() {
        Some(("/".to_string(), filename))
    } else {
        Some((parts.join("/"), filename))
    }
}
