fn main() {
    println!("{} {:?}", "Double:".to_string(), vec![1, 2, 3].into_iter().map(|x| (x * 2)).collect::<Vec<_>>());
    println!("{} {:?}", "Filter:".to_string(), vec![1, 10, 3].into_iter().filter(|&x| (x > 5)).collect::<Vec<_>>());
    println!("{} {}", "Sum:".to_string(), vec![1, 2, 3].into_iter().fold(0, |acc, x| (acc + x)));
}
