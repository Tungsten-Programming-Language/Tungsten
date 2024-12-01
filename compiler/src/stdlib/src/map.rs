use std::collections::HashMap;

/// Map/dictionary manipulation functions

/// Create a new map
pub fn new<K, V>() -> HashMap<K, V> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    HashMap::new()
}

/// Insert a key-value pair into a map
pub fn insert<K, V>(map: &mut HashMap<K, V>, key: K, value: V) 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.insert(key, value);
}

/// Get a value from a map by key
pub fn get<K, V>(map: &HashMap<K, V>, key: &K) -> Option<&V> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.get(key)
}

/// Remove a key-value pair from a map
pub fn remove<K, V>(map: &mut HashMap<K, V>, key: &K) -> Option<V> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.remove(key)
}

/// Get all keys from a map
pub fn keys<K, V>(map: &HashMap<K, V>) -> Vec<&K> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.keys().collect()
}

/// Get all values from a map
pub fn values<K, V>(map: &HashMap<K, V>) -> Vec<&V> 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.values().collect()
}

/// Check if a map contains a key
pub fn contains_key<K, V>(map: &HashMap<K, V>, key: &K) -> bool 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.contains_key(key)
}

/// Get the number of key-value pairs in a map
pub fn len<K, V>(map: &HashMap<K, V>) -> usize 
where 
    K: std::hash::Hash + Eq, 
    V: Clone 
{
    map.len()
}
