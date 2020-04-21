use std::collections::BTreeMap;

#[derive(Default)]
pub struct Counter<K>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    K: std::cmp::Ord,
{
    pub map: BTreeMap<K, u64>,
}

impl<K> Counter<K>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    K: std::cmp::Ord,
    K: std::default::Default,
{
    pub fn new() -> Counter<K> {
        Default::default()
    }

    pub fn record(&mut self, key: K) {
        if let Some(val) = self.map.get_mut(&key) {
            *val += 1;
        } else {
            self.map.insert(key, 1);
        }
    }
}
