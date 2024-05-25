use crate::{
    db::{Backend, Table},
    entry::{create_rand_work, LiteraryWork},
    utils::{decode_bincode, encode_bincode},
};

use anyhow::{bail, Result};
use rand::Rng;
use unicode_collate::collate;
use uuid::Uuid;

/// Name of the table within the DB
const WORKS_TABLE: &'static str = "WORKS";
/// Trash
const TRASH_TABLE: &'static str = "TRASH";

/// An abstraction over the backend to do library stuff. This allows for e.g. federated db access
pub struct Library<B>
where
    B: Backend,
{
    works: <B as Backend>::OutTable,
    trash: <B as Backend>::OutTable,
}

impl<B: Backend> Library<B> {
    /// Initialize a new instance of the abstraction using the database. It only opens the library table
    pub fn new(db: &B) -> Result<Self> {
        let works = db.get_table(WORKS_TABLE)?;
        let trash = db.get_table(TRASH_TABLE)?;
        Ok(Self { works, trash })
    }

    /// Add a work to the library
    // TODO: Add an API like HashMap's entry to update or incrementally update a Work (e.g. adding new chapter)
    pub fn add_work(&self, work: LiteraryWork) -> Result<()> {
        let uuid = Uuid::now_v7();
        self.works.insert(uuid, encode_bincode(&work)?.as_slice());
        Ok(())
    }

    /// Punts a work into the trash. Deleted works are never actually deleted
    // TODO: Allow admin to delete works permanently
    // TODO: Perhaps this can be done in terms of history?
    pub fn remove_work(&mut self, uuid: Uuid) {
        if let Some(work) = self.works.remove(uuid) {
            self.trash.insert(uuid, work);
        }
    }

    fn iter_works(&self) -> impl Iterator<Item = (Uuid, LiteraryWork)> + '_ {
        let works = self.works.iter();
        works.map(|(key, value)| {
            let id = Uuid::from_slice(key.as_ref()).unwrap();
            let work: LiteraryWork = decode_bincode(value.as_ref()).unwrap();
            (id, work)
        })
    }

    /// Returns a sorted vector
    pub fn all_works(&self) -> Vec<(Uuid, LiteraryWork)> {
        let mut res: Vec<_> = self.iter_works().collect();
        res.sort_unstable_by(|(_, a), (_, b)| collate(&a.title, &b.title));
        res
    }

    pub fn all_works_by(
        &self,
        f: impl Fn(&(Uuid, LiteraryWork)) -> bool,
    ) -> Vec<(Uuid, LiteraryWork)> {
        let mut res: Vec<_> = self.iter_works().filter(f).collect();
        res.sort_unstable_by(|(_, a), (_, b)| collate(&a.title, &b.title));
        res
    }

    pub fn get_work(&self, uuid: Uuid) -> Result<LiteraryWork> {
        let Some(data) = self.works.get_value(uuid) else {
            bail!("Could not find work!");
        };
        let work: LiteraryWork = decode_bincode(data.as_ref())?;
        Ok(work)
    }

    // TODO: Move to feature flag
    pub fn fill_test_data(&self) {
        let mut rng = rand::thread_rng();
        for _ in 0..rng.gen_range(10..100) {
            let w = create_rand_work();
            self.add_work(w).unwrap();
        }
    }
}
