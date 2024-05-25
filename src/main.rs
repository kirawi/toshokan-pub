mod config;
mod db;
mod error;
mod pages;
mod utils;

mod entry;
mod library;
mod stats;
mod user;

use std::sync::Arc;

use application::Application;

mod application;
mod params;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // println!(
    //     "{:x?}",
    //     unicode_collate::sort_key("\u{0627}\u{0591}\u{0655}\u{0061}")
    // );
    let state = Arc::new(Application::<sled::Db>::new("dev".into())?);
    // db::Backend::drop_table(&state.db, "WORKS")?;
    // db::Backend::drop_table(&state.db, "USERS")?;
    // state.lib.fill_test_data();

    tracing_subscriber::fmt::init();

    let app = routes::AppRoutes::register(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
