use std::{marker::PhantomData, sync::Arc};

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{application::Application, db::Backend, params, user};

// So I don't have to type generics everytime
pub struct AppRoutes<B: Backend> {
    _data: PhantomData<B>,
}

pub type App<B> = Arc<Application<B>>;

/*
    MVP Routes:
        - Home
        - Signup/login
        - Work
            - ToC/Info
                - Feat:
                    - Save to bookhself
                    - Personal rating
            - Chapters
        - Personal library
        - Directory (all works)
        - Shopping
            - Cart
            - Checkout
*/
impl<B: Backend + 'static> AppRoutes<B> {
    pub fn register(state: App<B>) -> Router {
        Router::new()
            .route("/", get(Self::home))
            .route("/works/:title/:id/:chapter_id", get(Self::get_chapter))
            .route("/works/:title/:id", get(Self::get_work))
            .route("/signup", get(Self::signup).post(Self::create_user))
            .route("/login", get(Self::login).post(Self::create_session))
            .route("/user", get(Self::user_library))
            .with_state(state)
    }

    /*
        - For all users, this will show the same page.
        - It contains:
            - Trending works
            - Recently updated works
            - Announcements
    */
    async fn home(State(state): State<App<B>>, jar: CookieJar) -> Html<String> {
        Html(state.home().unwrap())
    }

    async fn get_work(
        Path(params): Path<params::LiteraryWorkParams>,
        State(state): State<App<B>>,
    ) -> Html<String> {
        let Ok(work) = state.work(params) else {
            return Html("Work not found".to_string());
        };
        Html(work)
    }

    async fn get_chapter(
        Path(params): Path<params::ChapterParams>,
        State(state): State<App<B>>,
    ) -> Html<String> {
        // TODO: Handle
        let Ok(work) = state.chapter(params) else {
            return Html("Work not found".to_string());
        };
        Html(work)
    }

    async fn signup(State(state): State<App<B>>) -> Html<String> {
        Html(state.signup().unwrap())
    }

    async fn create_user(
        State(state): State<App<B>>,
        jar: CookieJar,
        Form(input): Form<params::CreateUserParams>,
    ) -> (CookieJar, Redirect) {
        let session = state
            .members
            .try_create_user(input.name, input.pswd)
            .unwrap();
        (
            jar.add(Cookie::new(
                user::SID_COOKIE,
                session.to_string().to_owned(),
            )),
            Redirect::to("/"),
        )
    }

    async fn login(State(state): State<App<B>>) -> Html<String> {
        Html(state.login().unwrap())
    }

    async fn create_session(
        State(state): State<App<B>>,
        jar: CookieJar,
        Form(input): Form<params::CreateUserParams>,
    ) -> (CookieJar, Redirect) {
        let session = state.members.login(input.name, input.pswd).unwrap();
        (
            jar.add(Cookie::new(
                user::SID_COOKIE,
                session.to_string().to_owned(),
            )),
            Redirect::to("/"),
        )
    }

    /*
        - For signed in users, this will show them:
            - Owned works
            - Active works (bookshelf)
        - Otherwise, it redirects to home
    */
    async fn user_library(State(state): State<App<B>>, jar: CookieJar) -> Response {
        let sid = jar.get(user::SID_COOKIE).and_then(|v| {
            let s = v.value_trimmed();
            uuid::Uuid::try_parse(s).ok()
        });
        if let Some(sid) = sid {
            match state.user_library(sid) {
                Ok(res) => Html(res).into_response(),
                Err(e) => e.to_string().into_response(),
            }
        } else {
            Redirect::to("/").into_response()
        }
    }
}
