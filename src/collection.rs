use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::LibraryError;

/// The overall container listing both the actively read books and the overall books in the library. Books being read are removed from the collection, and put back when finished.
#[derive(Default)]
pub struct Library {
    // TODO: Store position/page for current book
    active: HashSet<String>,
    /// Title: Content
    collection: BTreeMap<String, Content>,
}

/// The content of a book
struct Content {
    description: String,
    pages: Box<[String]>,
}

/// A [`Book`] consists of pages implemented via a list. It is created from both the title and `Content` in the library's btreemap
#[derive(Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub description: String,
    pub pages: Box<[String]>,
}

impl Library {
    /// Add a new [`Book`] to the library. If it already exists, [`LibraryError::Exists`] is returned.
    pub fn add(&mut self, bk: Book) -> Result<(), LibraryError> {
        let title = bk.title;
        let content = Content {
            description: bk.description,
            pages: bk.pages,
        };

        // Insert if it doesn't already exist, else return error
        match self.collection.entry(title.clone()) {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(content);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => Err(LibraryError::Exists(title)),
        }
    }

    // pub fn update(&mut self, bk: Book) {
    //     let title = bk.title;
    //     let content = Content {
    //         description: bk.description,
    //         pages: bk.pages,
    //     };

    //     self.collection
    //         .entry(title)
    //         .and_modify(|v| v = content)
    //         .or_insert(content);
    // }

    /// Add a book to the list of books currently being read
    pub fn read(&mut self, title: &str) -> Result<(), LibraryError> {
        if self.collection.get(&title.to_string()).is_none() {
            return Err(LibraryError::Missing(title.to_string()));
        }
        self.active.insert(title.to_string());
        Ok(())
    }
}
