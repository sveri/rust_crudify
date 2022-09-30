
use axum::body::Body;
use axum::http::Request;
use axum::{routing::get, routing::post, Extension, Router};

use axum::extract::{Json};

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{FromRow, Row, Pool, Postgres};

use uuid::Uuid;


#[derive(FromRow, Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Entity {
    id: Uuid,
    body: Value,
}

async fn create_entity(Extension(pool): Extension<PgPool>, body: Request<Body>) -> Json<Value> {
    
    let entity = Entity {id: Uuid::new_v4(), body: json!("asdflkj")};
    Json(json!(entity))
}

async fn create_table(table_name: &str, Extension(pool): Extension<PgPool>) {
    sqlx::query("CREATE TABLE IF NOT EXISTS $1").bind(table_name).execute(&pool).await;
}

async fn get_entities(Extension(pool): Extension<PgPool>) -> Json<Value> {
    let res = sqlx::query("SELECT id, body FROM entity")
        .fetch_all(&pool)
        .await
        .unwrap();
    let entities = res
        .into_iter()
        .map(|row| Entity {
            id: row.try_get(0).unwrap(),
            body: row.try_get(1).unwrap(),
        })
        .collect::<Vec<Entity>>();
    Json(json!(entities))
}

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

fn app(pool: Pool<Postgres>) -> Router {
    Router::new()
    .route("/api/entity", get(get_entities))
    .route("/api/entity", post(create_entity))
    .merge(axum_extra::routing::SpaRouter::new("/assets", "../dist"))
    .layer(Extension(pool))
}


#[cfg(test)] mod tests;