use std::time::SystemTime;

use anyhow::{bail, Result};
use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{Backend, Table},
    utils::{decode_bincode, encode_bincode},
};

const USER_TABLE: &'static str = "USERS";
pub const SID_COOKIE: &'static str = "session_id";

#[derive(Serialize, Encode, Decode)]
pub struct UserRef {
    pub name: String,
    pub created: SystemTime,
}

#[derive(Encode, Decode)]
pub struct UserData {
    #[bincode(with_serde)]
    pub created: DateTime<Utc>,
    // TODO: Use argon2
    pub pswd: String,

    // TODO: Expire after time and bound it to e.g. 100
    #[bincode(with_serde)]
    pub sessions: Vec<Uuid>,

    // Books in user library
    #[bincode(with_serde)]
    pub lib: UserLibrary,
}

#[derive(Default, Serialize, Deserialize)]
pub struct UserLibrary {
    /// Works owned/bought by the user
    pub works: Vec<Uuid>,
    /// Works currently being read and have associated data (current chapter, etc.). Not necessarily owned books
    /// e.g. trial or free books
    pub active: Vec<ActiveWork>,
}

#[derive(Serialize, Deserialize)]
pub struct ActiveWork {
    pub id: Uuid,
    pub read_chapters: Vec<Uuid>,
}

pub struct MemberCollection<B: Backend> {
    users: <B as Backend>::OutTable,
}

impl<B: Backend> MemberCollection<B> {
    pub fn new(db: &B) -> Result<Self> {
        let users = db.get_table(USER_TABLE)?;
        Ok(Self { users })
    }

    /// `name` and `pswd` are base64 encoded
    /// Returns session
    pub fn try_create_user(&self, name: String, pswd: String) -> Result<Uuid> {
        if self.users.get_value(&name).is_some() {
            bail!("User already exists");
        } else {
            let session = Uuid::now_v7();
            let user = UserData {
                created: Utc::now(),
                pswd,
                sessions: vec![session.clone()],
                lib: UserLibrary::default(),
            };
            self.users.insert(name, encode_bincode(&user)?);
            Ok(session)
        }
    }

    pub fn login(&self, name: String, pswd: String) -> Result<Uuid> {
        let Some(data) = self.users.get_value(&name) else {
            bail!("User doesn't exist!");
        };
        let user: UserData = decode_bincode(data.as_ref())?;

        // Borrow checker isn't smart enough
        std::mem::drop(data);

        if user.pswd == pswd {
            Ok(self.generate_session(name)?)
        } else {
            bail!("Wrong password!")
        }
    }

    fn generate_session(&self, name: String) -> Result<Uuid> {
        let session = Uuid::now_v7();
        let data = self.users.get_value(name.as_str()).unwrap();
        let mut user: UserData = decode_bincode(data.as_ref())?;

        // Borrow checker isn't smart enough
        std::mem::drop(data);

        user.sessions.push(session.clone());
        self.users.insert(name, encode_bincode(&user)?);
        Ok(session)
    }

    pub fn get_user_for_sid(&self, sid: Uuid) -> Option<String> {
        let row = self.users.iter().find(|(_, data)| {
            let UserData { sessions, .. } = decode_bincode(data.as_ref()).unwrap();
            if sessions.contains(&sid) {
                true
            } else {
                false
            }
        });
        row.map(|(name, _)| std::str::from_utf8(name.as_ref()).unwrap().to_string())
    }

    pub fn get_library(&self, name: &str) -> Result<UserLibrary> {
        let Some(data) = self.users.get_value(&name) else {
            bail!("User doesn't exist!");
        };
        let user: UserData = decode_bincode(data.as_ref())?;
        Ok(user.lib)
    }
}
