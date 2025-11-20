fn make_pair(x: i32, y: String) -> (i32, String) {
    (x, y)
}

fn get_first(pair: (i32, String)) -> (i32, String) {
    pair
}

fn main() {
    (1, "hello".to_string());
    (42, "world".to_string(), true);
    ();
    (42,);
    ((1, 2), (3, 4));
    ((1 + 2), (3 * 4), ((5 as i32).pow(2 as u32)));
    (vec![1, 2, 3], "list inside tuple".to_string());
    (10, "test".to_string());
    println!("{} {:?}", "Two-element tuple:".to_string(), (100, 200));
    println!("{} {:?}", "Three-element tuple:".to_string(), (42, "answer".to_string(), true));
    println!("{} {:?}", "Nested tuples:".to_string(), ((1, 2), (3, 4)));
}
