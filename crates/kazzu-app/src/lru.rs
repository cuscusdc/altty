//! Cache com limite de tamanho: ao passar da capacidade descarta o menos usado recentemente. Todas as operações são
//! O(1): um mapa chave → índice e uma lista duplamente encadeada em vetor (sem alocar por nó depois de cheio).

use std::collections::HashMap;
use std::hash::Hash;

const NIL: usize = usize::MAX;

struct Node<K, V> {
    key: K,
    value: V,
    prev: usize,
    next: usize,
}

pub struct Lru<K, V> {
    cap: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Option<Node<K, V>>>,
    free: Vec<usize>,
    /// Mais recente.
    head: usize,
    /// Menos recente (o próximo a sair).
    tail: usize,
}

impl<K: Hash + Eq + Clone, V> Lru<K, V> {
    pub fn new(cap: usize) -> Self {
        Self { cap: cap.max(1), map: HashMap::new(), nodes: Vec::new(), free: Vec::new(), head: NIL, tail: NIL }
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

    fn node(&self, i: usize) -> Option<&Node<K, V>> {
        self.nodes.get(i).and_then(Option::as_ref)
    }

    fn unlink(&mut self, i: usize) {
        let Some((prev, next)) = self.node(i).map(|n| (n.prev, n.next)) else { return };
        match prev {
            NIL => self.head = next,
            p => {
                if let Some(Some(n)) = self.nodes.get_mut(p) {
                    n.next = next;
                }
            }
        }
        match next {
            NIL => self.tail = prev,
            n_ => {
                if let Some(Some(n)) = self.nodes.get_mut(n_) {
                    n.prev = prev;
                }
            }
        }
    }

    fn push_front(&mut self, i: usize) {
        let old = self.head;
        if let Some(Some(n)) = self.nodes.get_mut(i) {
            n.prev = NIL;
            n.next = old;
        }
        if old != NIL
            && let Some(Some(n)) = self.nodes.get_mut(old)
        {
            n.prev = i;
        }
        self.head = i;
        if self.tail == NIL {
            self.tail = i;
        }
    }

    /// Lê sem marcar como usado.
    pub fn peek(&self, k: &K) -> Option<&V> {
        self.map.get(k).and_then(|&i| self.node(i)).map(|n| &n.value)
    }

    /// Lê e marca como usado agora.
    pub fn get(&mut self, k: &K) -> Option<&V> {
        let i = *self.map.get(k)?;
        self.unlink(i);
        self.push_front(i);
        self.node(i).map(|n| &n.value)
    }

    pub fn insert(&mut self, k: K, v: V) {
        if let Some(&i) = self.map.get(&k) {
            if let Some(Some(n)) = self.nodes.get_mut(i) {
                n.value = v;
            }
            self.unlink(i);
            self.push_front(i);
            return;
        }
        if self.map.len() >= self.cap && self.tail != NIL {
            let t = self.tail;
            self.unlink(t);
            if let Some(old) = self.nodes.get_mut(t).and_then(Option::take) {
                self.map.remove(&old.key);
            }
            self.free.push(t);
        }
        let node = Node { key: k.clone(), value: v, prev: NIL, next: NIL };
        let i = match self.free.pop() {
            Some(i) => {
                self.nodes[i] = Some(node);
                i
            }
            None => {
                self.nodes.push(Some(node));
                self.nodes.len() - 1
            }
        };
        self.map.insert(k, i);
        self.push_front(i);
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.free.clear();
        self.head = NIL;
        self.tail = NIL;
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

    #[test]
    fn overwrite_keeps_one_entry_and_refreshes_it() {
        let mut c = Lru::new(2);
        c.insert("a", 1);
        c.insert("b", 2);
        c.insert("a", 10); // a vira o mais recente
        c.insert("c", 3); // sai o b
        assert_eq!(c.peek(&"a"), Some(&10));
        assert!(!c.contains(&"b") && c.contains(&"c"));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn a_hundred_thousand_inserts_stay_fast_and_bounded() {
        // O(1) por operação: com a versão antiga (varredura a cada saída) isto levava minutos
        let mut c = Lru::new(4000);
        let t0 = std::time::Instant::now();
        for i in 0..100_000u32 {
            c.insert(i, i);
            if i % 7 == 0 {
                c.get(&(i / 2));
            }
        }
        assert_eq!(c.len(), 4000);
        assert!(t0.elapsed() < std::time::Duration::from_secs(2), "{:?}", t0.elapsed());
        c.clear();
        assert!(c.is_empty());
    }
}
