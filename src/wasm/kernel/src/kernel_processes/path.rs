pub fn join<T1, T2>(x: T1, y: T2) -> String
where
    T1: Into<String>,
    T2: Into<String>,
{
    let x = x.into();
    let y = y.into();

    if x.is_empty() {
        y
    } else if y.is_empty() {
        x
    } else if x.ends_with('/') {
        x + y.strip_prefix('/').unwrap_or(&y)
    } else {
        x + if !y.starts_with('/') { "/" } else { "" } + &y
    }
}

pub fn parent(path: &str) -> Option<String> {
    // /a/b/c -> /a/b
    // /a/b/c/ -> /a/b
    // / -> None
    // /a -> /

    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }

    let mut parts = path.split('/').collect::<Vec<_>>();
    parts.pop();

    if parts.is_empty() {
        Some("/".to_string())
    } else {
        Some(parts.join("/"))
    }
}

pub fn basename(path: &str) -> Option<String> {
    // /a/b/c -> c
    // /a/b/c/ -> c
    // / -> None
    // /a -> a

    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }

    let mut parts = path.split('/').collect::<Vec<_>>();
    Some(parts.pop().unwrap().to_string())
}
