use std::path::Path;

use anyhow::Result;

pub trait Table: Sized + Send + Sync {
    /// Iterator over all keys in the table
    fn keys(&self) -> impl Iterator<Item = impl AsRef<[u8]>>;

    /// Get a value in the table with the key
    fn get_value<K: AsRef<[u8]>>(&self, k: K) -> Option<impl AsRef<[u8]>>;

    /// Iterator over all values in the table
    fn values(&self) -> impl Iterator<Item = impl AsRef<[u8]>> {
        self.keys().map(|k| self.get_value(k)).flatten()
    }

    fn iter(&self) -> impl Iterator<Item = (impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        self.keys().zip(self.values())
    }

    /// If there are any keys in the table
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of keys in the table
    fn len(&self) -> usize {
        self.keys().count()
    }

    /// Insert (or update) a key with a new value. Returns the old value if it exists
    fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, k: K, v: V) -> Option<impl AsRef<[u8]>>;

    /// Removes a key-value pair if it exists
    fn remove<K: AsRef<[u8]>>(&self, k: K) -> Option<impl AsRef<[u8]>>;
}

/// Relational DB abstraction
// TODO: Add a way to delete a table
pub trait Backend: Sized + Send + Sync {
    /// Table
    type OutTable: Table;

    /// Open DB from path
    // TODO: Switch to a URI instead
    fn open(p: impl AsRef<Path>) -> Result<Self>;

    /// Iterate over all table names
    fn tables(&self) -> Vec<String>;

    /// table
    fn get_table(&self, table: &str) -> Result<Self::OutTable>;

    /// Permanently erases a table
    fn drop_table(&self, table: &str) -> Result<()>;
}

pub mod sled_backend {
    use sled::{Db, Tree};

    use super::*;

    impl Table for Tree {
        fn keys(&self) -> impl Iterator<Item = impl AsRef<[u8]>> {
            self.iter().keys().map(|k| k.unwrap())
        }

        fn get_value<K: AsRef<[u8]>>(&self, k: K) -> Option<impl AsRef<[u8]>> {
            self.get(k).ok().flatten()
        }

        fn is_empty(&self) -> bool {
            Tree::is_empty(&self)
        }

        fn len(&self) -> usize {
            Tree::len(&self)
        }

        fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, k: K, v: V) -> Option<impl AsRef<[u8]>> {
            Tree::insert(&self, k, v.as_ref()).ok().flatten()
        }

        fn remove<K: AsRef<[u8]>>(&self, k: K) -> Option<impl AsRef<[u8]>> {
            Tree::remove(&self, k).ok().flatten()
        }
    }

    impl Backend for Db {
        type OutTable = Tree;

        fn open(p: impl AsRef<Path>) -> Result<Self> {
            let db = sled::open(p)?;
            Ok(db)
        }

        fn tables(&self) -> Vec<String> {
            self.tree_names()
                .iter()
                .map(|v| std::str::from_utf8(v).unwrap().to_string())
                .collect()
        }

        fn get_table(&self, table: &str) -> Result<Self::OutTable> {
            Ok(self.open_tree(table)?)
        }

        fn drop_table(&self, table: &str) -> Result<()> {
            Db::drop_tree(&self, table)?;
            Ok(())
        }
    }
}
