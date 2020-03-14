use std::collections::HashMap;

pub struct Counter<K>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
{
    pub map: HashMap<K, u64>,
}

impl<K> Counter<K>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
{
    pub fn new() -> Counter<K> {
        return Counter {
            map: HashMap::new(),
        };
    }

    pub fn record(&mut self, key: K) {
        if let Some(val) = self.map.get_mut(&key) {
            *val += 1;
        } else {
            self.map.insert(key, 1);
        }
    }
}
