use serde::Deserialize;
use uuid::Uuid;

use crate::utils::b64_decode_uuid;

#[derive(Deserialize)]
pub struct LiteraryWorkParams {
    pub title: String,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct ChapterParams {
    #[serde(flatten)]
    pub work_params: LiteraryWorkParams,
    pub chapter_id: usize,
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = b64_decode_uuid(&s);
    if let Ok(uuid) = Uuid::from_slice(bytes.as_ref()) {
        Ok(uuid)
    } else {
        Err(serde::de::Error::custom("Invalid UUID"))
    }
}

#[derive(Deserialize)]
pub struct CreateUserParams {
    pub name: String,
    pub pswd: String,
}
