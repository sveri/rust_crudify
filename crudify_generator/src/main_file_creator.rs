use std::{
    fs::{self, File},
    io::{self, Write},
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
use serde_json::{json, Value};
 "#
}

fn get_routing_functions_code(models: &InternalModels) -> String {
    let mut code = "".to_string();

    for model in models.iter() {
        code.push_str(&format!("async fn get_{}(Extension(pool): Extension<PgPool>) -> Json<Value> {{", model.name.to_lowercase()));
        let fn_code = r#"
            let res = sqlx::query("SELECT id, body FROM entity").fetch_all(&pool).await.unwrap();
            let entities = res
                .into_iter()
                .map(|row| Entity {
                    id: row.try_get(0).unwrap(),
                    body: row.try_get(1).unwrap(),
                })
                .collect::<Vec<Entity>>();
            Json(json!(entities))
        }
        
        "#;
        code.push_str(fn_code);
    }

    code.to_string()
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
    for model in models.iter() {
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

fn create_or_get_src_dir(user_id: &str) -> Result<PathBuf, io::Error> {
    let current_dir = std::env::current_dir()?;
    let data_path = current_dir.join("../").join(user_id).join("src");
    fs::create_dir_all(&data_path)?;
    Ok(data_path)
}

pub fn write_main_file(user_id: &str, models: &InternalModels) -> Result<(), io::Error> {
    let code = format!("{}\n\n {} {} {}", get_usages(), get_routing_functions_code(models), get_main_fn_code(), create_app_fn(models));

    let data_path = create_or_get_src_dir(user_id)?.join("main.rs");
    let mut main_rs = File::create(data_path)?;

    main_rs.write_all(code.as_bytes())?;
    Ok(())
}
