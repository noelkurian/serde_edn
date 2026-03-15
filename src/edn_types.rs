use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdnList<T> {
    pub items: Vec<T>,
}

impl<T> EdnList<T> {
    pub fn new() -> Self {
        EdnList { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        EdnList {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        EdnList { items }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

impl<T> Default for EdnList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for EdnList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for EdnList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T> Index<usize> for EdnList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<usize> for EdnList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl<T> IntoIterator for EdnList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a EdnList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<T> From<Vec<T>> for EdnList<T> {
    fn from(items: Vec<T>) -> Self {
        EdnList { items }
    }
}

impl<T> From<EdnList<T>> for Vec<T> {
    fn from(list: EdnList<T>) -> Self {
        list.items
    }
}

impl<T: Serialize> Serialize for EdnList<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("__edn_list__", &self.items)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for EdnList<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EdnListVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for EdnListVisitor<T> {
            type Value = EdnList<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an EDN list")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(EdnList { items })
            }
        }

        deserializer
            .deserialize_newtype_struct("__edn_list__", EdnListVisitor(std::marker::PhantomData))
    }
}

#[derive(Clone, Debug)]
pub struct EdnSet<T> {
    pub items: HashSet<T>,
}

impl<T> EdnSet<T> {
    pub fn new() -> Self {
        EdnSet {
            items: HashSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        EdnSet {
            items: HashSet::with_capacity(capacity),
        }
    }

    pub fn from_hashset(items: HashSet<T>) -> Self {
        EdnSet { items }
    }

    pub fn into_hashset(self) -> HashSet<T> {
        self.items
    }

    pub fn insert(&mut self, value: T) -> bool
    where
        T: Hash + Eq,
    {
        self.items.insert(value)
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Hash + Eq + std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.items.contains(value)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for EdnSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for EdnSet<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for EdnSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T> IntoIterator for EdnSet<T> {
    type Item = T;
    type IntoIter = std::collections::hash_set::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a EdnSet<T> {
    type Item = &'a T;
    type IntoIter = std::collections::hash_set::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<T: Hash + Eq> From<HashSet<T>> for EdnSet<T> {
    fn from(items: HashSet<T>) -> Self {
        EdnSet { items }
    }
}

impl<T: Hash + Eq> From<EdnSet<T>> for HashSet<T> {
    fn from(set: EdnSet<T>) -> Self {
        set.items
    }
}

impl<T: Hash + Eq + Serialize> Serialize for EdnSet<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let items: Vec<&T> = self.items.iter().collect();
        serializer.serialize_newtype_struct("__edn_set__", &items)
    }
}

impl<'de, T: Deserialize<'de> + Hash + Eq> Deserialize<'de> for EdnSet<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EdnSetVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de> + Hash + Eq> Visitor<'de> for EdnSetVisitor<T> {
            type Value = EdnSet<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an EDN set")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items = HashSet::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element()? {
                    items.insert(item);
                }
                Ok(EdnSet { items })
            }
        }

        deserializer
            .deserialize_newtype_struct("__edn_set__", EdnSetVisitor(std::marker::PhantomData))
    }
}
