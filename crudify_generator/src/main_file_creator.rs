use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::InternalModels;

fn get_usages<'a>() -> &'a str {
    r#"use axum::{
    body::Body,
    extract::Json,
    http::Request,
    routing::{delete, get, post, put},
    Extension, Router,    
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, PgPool};
use serde_json::json;
 "#
    }

fn get_main_fn_code<'a>() -> &'a str {
    r#"

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("cannot connect to database");

    axum::Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app(pool).into_make_service())
        .await
        .unwrap();
}

    "#
}

fn create_app_fn(models: &InternalModels) -> String {
    let mut code = r#"
fn app(pool: Pool<Postgres>) -> Router {
    Router::new()
"#
    .to_string();
    for model in models.into_iter() {
        let name = model.name.to_lowercase();
        code.push_str(format!(".route(\"/api/{0}\", post(post_{0}))\n", name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", put(put_{0}))\n", name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", get(get_{0}))\n", name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", delete(delete_{0}))\n", name).as_str());
    }
    code.push_str(
        r#".merge(axum_extra::routing::SpaRouter::new("/assets", "../dist"))
    .layer(Extension(pool))
}"#,
    );

    code
}

fn create_or_get_src_dir(user_id: &str) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let data_path = current_dir.join("../").join(user_id).join("src");
    fs::create_dir_all(&data_path).unwrap();
    data_path
}

pub fn write_main_file(user_id: &str, models: &InternalModels) {
    let code = format!("{} {} {}", get_usages(), get_main_fn_code(), create_app_fn(models));

    let data_path = create_or_get_src_dir(user_id).join("main.rs");
    let mut main_rs = File::create(data_path).unwrap();

    main_rs.write_all(code.as_bytes()).unwrap();
}
