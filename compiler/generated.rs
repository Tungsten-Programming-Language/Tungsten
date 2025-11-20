fn main() {
    match 42 {
    _ => "matches anything".to_string(),
};
    match 5 {
    1 => "one".to_string(),
    2 => "two".to_string(),
    3 => "three".to_string(),
    _ => "other number".to_string(),
};
    match 100 {
    x => x,
};
    match "hello".to_string() {
    msg => msg,
};
    match true {
    true => "yes".to_string(),
    false => "no".to_string(),
};
    match Some(42) {
    Some(x) => x,
    None => 0,
};
    match Some(Some(100)) {
    Some(Some(val)) => val,
    Some(None) => 0,
    None => 0,
};
    match Some(42) {
    Some(value) => value,
    None => 0,
};
    match (1, 2) {
    (x, y) => x,
};
    match (10, "hello".to_string(), true) {
    (num, str, bool) => num,
};
    match ((1, 2), (3, 4)) {
    ((a, b), (c, d)) => a,
};
    match (Some(5), Some(10)) {
    (Some(x), Some(y)) => x,
    (None, _) => 0,
    (_, None) => 0,
};
    match vec![1, 2, 3] {
    _ => "any list".to_string(),
};
    match Some((42, "answer".to_string())) {
    Some((num, str)) => num,
    None => 0,
};
    match Some(Some((1, 2))) {
    Some(Some((x, y))) => x,
    Some(None) => 0,
    None => 0,
};
    println!("{} {}", "Wildcard match:".to_string(), match 99 {
    _ => "anything".to_string(),
});
    println!("{} {}", "Number match:".to_string(), match 7 {
    7 => "lucky seven".to_string(),
    13 => "unlucky thirteen".to_string(),
    _ => "some other number".to_string(),
});
    println!("{} {}", "Option match:".to_string(), match Some(42) {
    Some(x) => x,
    None => 0,
});
    println!("{} {}", "Option match (nested):".to_string(), match Some(Some(100)) {
    Some(Some(val)) => val,
    _ => 0,
});
    println!("{} {}", "Tuple match:".to_string(), match (100, 200) {
    (a, b) => a,
});
    match Some(10) {
    Some(result) => result,
    None => 0,
};
    match Some("config.json".to_string()) {
    Some(filename) => filename,
    None => "default.json".to_string(),
};
    match 200 {
    200 => "OK".to_string(),
    404 => "Not Found".to_string(),
    500 => "Internal Server Error".to_string(),
    _ => "Unknown Status".to_string(),
};
}
