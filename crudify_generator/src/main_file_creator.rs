use crate::sql_creator::{create_get_all_entities, create_create_entity, create_update_entity};
use crate::InternalModels;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

fn get_usages<'a>() -> &'a str {
    r#"
use std::{
    fmt::{Display, Formatter},
};
use axum::{
    extract::{Json, Path},
    http::{StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Router,    
};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, PgPool, FromRow};
use thiserror::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
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
        code.push_str(&format!(
            "#[derive(FromRow, Serialize, Deserialize)]\nstruct {} {{\n{}\n}}",
            model.name, properties
        ))
    }

    code
}

fn get_routing_functions_code(models: &InternalModels) -> String {
    let mut code = "".to_string();

    for model in models.iter() {
        code.push_str(&format!(
            "async fn get_{}(Extension(pool): Extension<PgPool>) -> Result<Json<Value>, AppError> {{\n
                let res: Vec<{}> = sqlx::query_as(\"{}\").fetch_all(&pool).await?;
                Ok(Json(json!(res)))
        }}\n",
            model.name.to_lowercase(), model.name, create_get_all_entities(model)
        ));

        let binds: String = match &model.properties {
            None => "".to_string(),
            Some(properties) => {
                properties.keys().into_iter().map(|k| format!(".bind(&{}.{})", model.name.to_lowercase(), k)).collect()
            }
        };

        code.push_str(
            &format!(r#"
            async fn post_{}(Json({0}): Json<{}>, Extension(pool): Extension<PgPool>) -> Result<Json<Value>, AppError> {{
                let query = "{}";
                sqlx::query(query){}.execute(&pool).await?;
                Ok(Json(json!({0})))
            }}"#, model.name.to_lowercase(), model.name, create_create_entity(model), binds));

            

        // let binds_without_id = match &model.properties {
        //     None => "".to_string(),
        //     Some(properties) => {
        //         properties.keys().into_iter().map(|k| k != "id" ? format!(".bind(&{}.{})", model.name.to_lowercase(), k)).collect()
        //     }
        // };

        code.push_str(&format!(
                r#"    async fn put_{}(Path(id): Path<i64>, Json({0}): Json<{}>, Extension(pool): Extension<PgPool>) -> Result<Json<Value>, AppError> {{
                let query = "{}";
                sqlx::query(query).bind(id).bind(&order.name).execute(&pool).await?;
                let o: Order = order;
                Ok(Json(json!(o)))
            }}"#, model.name.to_lowercase(), model.name, create_update_entity(model)));

        code.push_str(r#"
            async fn delete_order(Path(id): Path<i64>, Extension(pool): Extension<PgPool>) -> Result<(), AppError> {
                let query = "DELETE FROM public.order WHERE id = $1";
                sqlx::query(query).bind(id).execute(&pool).await?;
                Ok(())
            }
            "#,
        );
    }

    code.to_string()
}

fn get_main_fn_code<'a>() -> &'a str {
    r#"

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("cannot connect to database");

    match create_tables(&pool).await {
        Ok(_) => println!("Created tables"),
        Err(e) => eprintln!("Error while creating database tables: {:#}", e)
    }

    axum::Server::bind(&"127.0.0.1:8000".parse().expect("Expected a parsable URI to start a server on"))
        .serve(app(pool).into_make_service())
        .await
        .expect("Could not start server");
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
        code.push_str(format!(".route(\"/api/{0}/:id\", put(put_{0}))\n", name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", get(get_{0}))\n", name).as_str());
        code.push_str(format!(".route(\"/api/{0}/:id\", delete(delete_{0}))\n", name).as_str());
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
async fn create_tables(pool: &Pool<Postgres>) -> Result<(), AppError> {
    let query = "CREATE TABLE IF NOT EXISTS public.order (id bigint NULL, name text NULL);";
    sqlx::query(query).execute(pool).await?;
    Ok(())
}    
    "#
    .to_string()
}

fn get_error_setup() -> String {
    r#"
#[derive(Serialize, Debug, Error)]
pub struct AppError {
    status_code: u16,
    errors: Vec<String>,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Err {} ", &self.status_code))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("SQL error: {:?}", e);
        AppError::new_internal(e.to_string())
    }
}

impl AppError {
    pub fn new(status_code: u16, err: String) -> Self {
        AppError {
            status_code,
            errors: vec![err],
        }
    }

    pub fn new_internal(err: String) -> Self {
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            errors: vec![err],
        }
    }
    pub fn new_bad_request(err: String) -> Self {
        AppError {
            status_code: StatusCode::BAD_REQUEST.as_u16(),
            errors: vec![err],
        }
    }

    pub fn append_error(&mut self, err: String) {
        let _ = &self.errors.push(err);
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.status_code).unwrap(),
            serde_json::to_string(&self).unwrap(),
        )
            .into_response()
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
