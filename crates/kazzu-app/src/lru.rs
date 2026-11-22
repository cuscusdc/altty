//! Cache com limite de tamanho: ao passar da capacidade descarta o menos usado recentemente.

use std::collections::HashMap;
use std::hash::Hash;

pub struct Lru<K, V> {
    cap: usize,
    tick: u64,
    map: HashMap<K, (V, u64)>,
}

impl<K: Hash + Eq + Clone, V> Lru<K, V> {
    pub fn new(cap: usize) -> Self {
        Self { cap: cap.max(1), tick: 0, map: HashMap::new() }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains(&self, k: &K) -> bool {
        self.map.contains_key(k)
    }

    /// Lê sem marcar como usado.
    pub fn peek(&self, k: &K) -> Option<&V> {
        self.map.get(k).map(|e| &e.0)
    }

    /// Lê e marca como usado agora.
    pub fn get(&mut self, k: &K) -> Option<&V> {
        self.tick += 1;
        let t = self.tick;
        self.map.get_mut(k).map(|e| {
            e.1 = t;
            &e.0
        })
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.tick += 1;
        self.map.insert(k, (v, self.tick));
        while self.map.len() > self.cap {
            let Some(oldest) = self.map.iter().min_by_key(|(_, (_, t))| *t).map(|(k, _)| k.clone()) else { break };
            self.map.remove(&oldest);
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_the_least_recently_used_and_never_exceeds_capacity() {
        let mut c = Lru::new(3);
        for i in 0..3 {
            c.insert(i, i * 10);
        }
        assert_eq!(c.get(&0), Some(&0)); // 0 passa a ser o mais recente; 1 é o mais antigo
        c.insert(3, 30);
        assert_eq!(c.len(), 3);
        assert!(!c.contains(&1));
        assert!(c.contains(&0) && c.contains(&2) && c.contains(&3));
        for i in 10..1000 {
            c.insert(i, 0);
        }
        assert_eq!(c.len(), 3);
    }
}
