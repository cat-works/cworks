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
        x + if y.starts_with('/') { &y[1..] } else { &y }
    } else {
        x + if !y.starts_with('/') { "/" } else { "" } + &y
    }
}
