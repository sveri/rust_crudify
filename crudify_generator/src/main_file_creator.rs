use crate::sql_creator::create_get_all_entities;
use crate::InternalModels;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

fn get_usages<'a>() -> &'a str {
    r#"
use anyhow::{Context, Result as AnyResult};
use axum::{
    body::Body,
    extract::Json,
    http::{StatusCode, Request},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Router,    
};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, PgPool, Row};
 "#
}

fn get_structs(models: &InternalModels) -> String {
    let mut code = "".to_string();

    for model in models.iter() {
        let properties: String = match &model.properties {
            None => "".to_string(),
            Some(properties) => {
                let mut props_string: String = "".to_string();
                for (key, value) in properties {
                    props_string.push_str(&format!("{}: {},\n", key, value));
                }
                props_string
            }
        };
        code.push_str(&format!("#[derive(Serialize, Deserialize)]\nstruct {} {{\n{}\n}}", model.name, properties))
    }

    code
}

fn get_routing_functions_code(models: &InternalModels) -> String {
    let mut code = "".to_string();

    for model in models.iter() {
        code.push_str(&format!(
            "async fn get_{}(Extension(pool): Extension<PgPool>) -> Json<Value> {{\n",
            model.name.to_lowercase()
        ));

        code.push_str(&format!(
            "let res = sqlx::query(\"{}\").fetch_all(&pool).await.unwrap();",
            create_get_all_entities(model)
        ));

        let properties_string: String = match &model.properties {
            None => "".to_string(),
            Some(properties) => properties
                .keys()
                .map(|k| {
                    format!(
                        "{}: row.try_get({:?}).unwrap()",
                        k,
                        properties.get_index_of(k).expect("IndexMap did not return an index for key.")
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n"),
        };

        let fn_code = format!(
            r#"
            let entities = res
                .into_iter()
                .map(|row| {} {{
                    {}
                }})
                .collect::<Vec<{0}>>();
            Json(json!(entities))
        }}

        "#,
            model.name, properties_string
        )
        .to_string();
        code.push_str(&fn_code);

        code.push_str(r#"

        async fn post_order(order: Json<Order>, Extension(pool): Extension<PgPool>) -> Result<Json<Value>, AppError>{
            let query = "INSERT INTO order (id, name) VALUES ($1, $2)";
            let res = sqlx::query(query)
                .bind(order.id)
                .bind(&order.name)
                .execute(&pool)
                .await
                .context(format!("Could not execute SQL: {}", query))?;
            Ok(Json(json!("order")))
        }"#);
    }

    code.to_string()
}

fn get_main_fn_code<'a>() -> &'a str {
    r#"

#[tokio::main]
async fn main() -> AnyResult<()> {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("cannot connect to database");

    match create_tables(&pool).await {
        Ok(_) => println!("Created tables"),
        Err(e) => eprintln!("Error while creating database tables: {:#}", e)
    }
    

    axum::Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app(pool).into_make_service())
        .await
        .unwrap();
    Ok(())
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

fn create_sql_create_tables(models: &InternalModels) -> String {
    r#"
async fn create_tables(pool: &Pool<Postgres>) -> AnyResult<()> {
    let query = "CREATE TABLE IF NOT EXISTS public.order (id bigint NULL, name text NULL);";
    sqlx::query(query)
        .execute(pool)
        .await
        .context(format!("Could not execute SQL: {}", query))?;
    Ok(())
}
    "#
    .to_string()
}

fn get_error_setup() -> String {
    r#"
enum AppError {
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(inner: anyhow::Error) -> Self {
        AppError::InternalServerError(inner)
    }
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(inner) => {
                
                (StatusCode::INTERNAL_SERVER_ERROR, inner.to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}    
    "#
    .to_string()
}

fn create_or_get_src_dir(user_id: &str) -> Result<PathBuf, io::Error> {
    let current_dir = std::env::current_dir()?;
    let data_path = current_dir.join("../").join(user_id).join("src");
    fs::create_dir_all(&data_path)?;
    Ok(data_path)
}

pub fn write_main_file(user_id: &str, models: &InternalModels) -> Result<(), io::Error> {
    let code = format!(
        "{}\n\n {}\n\n {} {} {}\n {}\n {}",
        get_usages(),
        get_structs(models),
        get_routing_functions_code(models),
        get_main_fn_code(),
        create_app_fn(models),
        create_sql_create_tables(models),
        get_error_setup()
    );

    let data_path = create_or_get_src_dir(user_id)?.join("main.rs");
    let mut main_rs = File::create(data_path)?;

    main_rs.write_all(code.as_bytes())?;
    Ok(())
}
