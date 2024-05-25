use anyhow::{bail, Result};
use minijinja::{context, Environment, Value};
use uuid::Uuid;

use crate::{
    db::Backend, library::Library, params, user::MemberCollection, utils::b64_encode_uuid,
};

/// Application State:
/// - User data
///   - Users
///   - Currently reading books
///   - Book progress
/// - Global library data
///   - All books
///   - Statistics (word count, people reading it, rating)
///   - Books releasing soon
pub struct Application<B: Backend> {
    pub db: B,
    /// Contains library data
    pub lib: Library<B>,
    /// User data
    pub members: MemberCollection<B>,

    env: Environment<'static>,
}

impl<B: Backend> Application<B> {
    /// Opens the db at `uri`
    pub fn new(p: std::path::PathBuf) -> Result<Self> {
        let db = B::open(p)?;
        let lib = Library::new(&db)?;
        let members = MemberCollection::new(&db)?;
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("templates"));

        Ok(Self {
            db,
            lib,
            members,
            env,
        })
    }

    /// Render the homepage
    pub fn home(&self) -> Result<String> {
        let template = self.env.get_template("home.jinja")?;
        let all_works = self.lib.all_works();

        // // Homepage shown to everyone
        // // Shows recent books, top-rated books, etc.
        // if let Some(name) = session_id.and_then(|sid| self.members.get_user_for_sid(sid)) {
        //     let lib = self.members.get_library(&name)?;
        //     let works = self.lib.all_works_by(|(id, _)| lib.works.contains(id));
        //     let active_works = self
        //         .lib
        //         .all_works_by(|(id, _)| lib.active.iter().find(|aw| aw.id == *id).is_some());
        //     let template = self.env.get_template("userhome.jinja")?;
        // } else {
        // }

        let iter = all_works.into_iter().map(|(id, work)| {
            let b64_id = b64_encode_uuid(id.as_bytes());
            let title = work.title;
            context! { title, uuid => b64_id }
        });
        let templ_works = Value::from_iter(iter);

        // By now, all the data should have been fetched, and so we can render the template
        let render = template.render(context! { collection => templ_works })?;
        Ok(render)
    }

    pub fn work(&self, params: params::LiteraryWorkParams) -> Result<String> {
        let template = self.env.get_template("work.jinja")?;
        let work = self.lib.get_work(params.id)?;

        // Now, get all the chapters
        let iter = work.chapters.into_iter().enumerate().map(|(id, chapter)| {
            context! {
                id,
                title => chapter.title,
                date => chapter.date.date_naive()
            }
        });
        let chapters = Value::from_iter(iter);

        let uuid = b64_encode_uuid(params.id.as_bytes());
        // By now, all the data should have been fetched, and so we can render the template
        let render = template.render(context! { uuid, title => work.title, description => work.description, creators => work.creators, chapters => chapters })?;
        Ok(render)
    }

    /// Render the chapter of a work
    pub fn chapter(&self, params: params::ChapterParams) -> Result<String> {
        let template = self.env.get_template("chapter.jinja")?;
        let mut work = self.lib.get_work(params.work_params.id)?;

        // Out of bounds
        if params.chapter_id >= work.chapters.len() {
            bail!("Could not find chapter!");
        }
        let chapter = work.chapters.remove(params.chapter_id);

        // TODO: Handle images
        let iter = chapter.elements.into_iter().map(|e| match e {
            crate::entry::Entry::Paragraph(p) => p,
            crate::entry::Entry::Image(_) => todo!(),
        });
        let entries = Value::from_iter(iter);

        // By now, all the data should have been fetched, and so we can render the template
        let render = template.render(context! { work_title => params.work_params.title, chapter_title => chapter.title, entries })?;
        Ok(render)
    }

    pub fn signup(&self) -> Result<String> {
        let template = self.env.get_template("signup.jinja")?;
        let render = template.render(context! {})?;
        Ok(render)
    }

    pub fn login(&self) -> Result<String> {
        let template = self.env.get_template("login.jinja")?;
        let render = template.render(context! {})?;
        Ok(render)
    }

    pub fn user_library(&self, sid: Uuid) -> Result<String> {
        if let Some(name) = self.members.get_user_for_sid(sid) {
            let lib = self.members.get_library(&name)?;

            // Iterate through the global library to find the user's works' metadata
            let owned_works = self.lib.all_works_by(|(id, _)| lib.works.contains(id));
            let active_works = self
                .lib
                .all_works_by(|(id, _)| lib.active.iter().find(|aw| aw.id == *id).is_some());
            let template = self.env.get_template("userhome.jinja")?;
            let render = template.render(context! { owned_works, active_works })?;
            Ok(render)
        } else {
            bail!("User does not exist!")
        }
    }
}
