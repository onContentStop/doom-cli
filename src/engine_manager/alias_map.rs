use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub struct AliasMap<K, V>
where
    K: Eq + Hash,
{
    aliases: HashMap<K, usize>,
    data: Vec<V>,
}

impl<K, V> AliasMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        AliasMap {
            aliases: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn with_capacities(key_count: usize, value_count: usize) -> Self {
        AliasMap {
            aliases: HashMap::with_capacity(key_count),
            data: Vec::with_capacity(value_count),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> &V {
        let data_index = self.data.len();
        self.data.push(value);
        self.aliases.insert(key, data_index);
        &self.data[data_index]
    }

    pub fn alias<Q>(&mut self, previous_key: &Q, new_key: K) -> Result<(), &'static str>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let data_index = *self
            .aliases
            .get(previous_key)
            .ok_or("Previous key does not exist.")?;
        self.aliases.insert(new_key, data_index);
        Ok(())
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.aliases.get(key).map(|&i| &self.data[i])
    }
}

#[cfg(test)]
mod tests {
    use super::AliasMap;

    #[test]
    fn alias_map_empty() {
        let map = AliasMap::<String, u8>::new();
        assert_eq!(map.get("test"), None);
    }

    #[test]
    fn alias_map_retrieve() {
        let mut map = AliasMap::new();
        map.insert(String::from("test"), 0);
        assert_eq!(map.get("test"), Some(&0));
        assert_eq!(map.get("set"), None);
    }

    #[test]
    fn alias_map_retrieve_aliased() {
        let mut map = AliasMap::new();
        map.insert(String::from("test"), 0);
        map.alias("test", String::from("set")).unwrap();
        assert_eq!(map.get("test").unwrap(), map.get("set").unwrap());
    }
}
