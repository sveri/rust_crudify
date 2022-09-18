use axum::{routing::get, Extension, Router};
use axum::body::{boxed, Body};
use axum::http::{Response, StatusCode};

use axum::extract::{Json, Path, Query};

use serde::Serialize;
use serde_json::{json, Value};

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{FromRow, Row};

use uuid::Uuid;

use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;

#[derive(FromRow, Clone, Debug, Serialize)]
struct Entity {
    id: Uuid,
    body: Value,
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
        .expect("can connect to database");

    // build our application with a single route
    let app = Router::new()
        // .route("/", get(|| async { "Hello, Worasdfld!" }))
        .route("/api/entity", get(get_entities))
        .merge(axum_extra::routing::SpaRouter::new("/assets", "../dist"))
        // .fallback(get(|req| async move {
        //     match ServeDir::new("../dist").oneshot(req).await {
        //         Ok(res) => res.map(boxed),
        //         Err(err) => Response::builder()
        //             .status(StatusCode::INTERNAL_SERVER_ERROR)
        //             .body(boxed(Body::from(format!("error: {err}"))))
        //             .expect("error response"),
        //     }
        // }))
        .layer(Extension(pool));

    axum::Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
