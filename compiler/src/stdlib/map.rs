/// Map manipulation functions for the standard library

use std::collections::HashMap;

/// Create a new empty HashMap
pub fn new<K, V>() -> HashMap<K, V> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    HashMap::new()
}

/// Insert a key-value pair into a HashMap
pub fn insert<K, V>(map: &mut HashMap<K, V>, key: K, value: V)
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.insert(key, value);
}

/// Get a value from a HashMap by key
pub fn get<'a, K, V>(map: &'a HashMap<K, V>, key: &'a K) -> Option<&'a V>
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.get(key)
}
