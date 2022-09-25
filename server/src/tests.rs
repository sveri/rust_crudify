use crate::{app, Entity};


use axum::{ Error,
    response::Response,
    body::Body, body::Bytes,
    http::{Request, StatusCode, Method},
    Router,
};

use http_body::combinators::UnsyncBoxBody;

use serde_json::{json};
use sqlx::postgres::{PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;




async fn setup() -> Router {
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("cannot connect to database");

    app(pool)
}

async fn post<T>(app: Router, uri: &str, entity: &T) -> Response<UnsyncBoxBody<Bytes, Error>> 
where T: serde::Serialize {
    app.oneshot(Request::builder().method(Method::POST).uri(uri).body(Body::from(serde_json::to_vec(&entity).unwrap())).unwrap().into()).await.unwrap()
}

async fn response_to_entity<T> (r: Response<UnsyncBoxBody<Bytes, Error>>) -> T 
where T: serde::de::DeserializeOwned
    {
    let body = hyper::body::to_bytes(r.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn get_entities() {    
    let app = setup().await;

    let res = app.oneshot(Request::builder().uri("/api/entity").body(Body::empty()).unwrap().into()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_entity() {    
    let app = setup().await;

    let entity = Entity {id: Uuid::new_v4(), body: json!("asdfqweoijrqweorjlkj")};

    let res = post(app, "/api/entity", &entity).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res_entity: Entity = response_to_entity(res).await;
    println!("++++++++++++++++{:?}", res_entity);

    assert_eq!(entity, res_entity);
}
