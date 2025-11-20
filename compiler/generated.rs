#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub age: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
}

fn main() {
    println!("{} {:?}", "Point:".to_string(), Point { x: 10, y: 20 });
    println!("{} {:?}", "Person:".to_string(), Person { name: "Alice".to_string(), age: 30 });
    println!("{} {:?}", "Rectangle:".to_string(), Rectangle { width: 100, height: 50, x: 0, y: 0 });
    println!("{} {:?}", "Computed Point:".to_string(), Point { x: (5 + 5), y: (10 * 2) });
    println!("{} {:?}", "Dynamic Rectangle:".to_string(), Rectangle { width: (10 * 2), height: (5 + 10), x: 0, y: 0 });
}
