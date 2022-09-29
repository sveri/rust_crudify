use std::{fs, io::Write, path::PathBuf};

use super::InternalModels;

fn create_app_fn(models: InternalModels) -> String {
    let mut code = r#"
fn app(pool: Pool<Postgres>) -> Router {
    Router::new()
    "#
    .to_string();
    for model in models.into_iter() {
        code.push_str(format!(".route(\"/api/{0}\", post(post_{0}))", model.name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", put(put_{0}))", model.name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", get(get_{0}))", model.name).as_str());
        code.push_str(format!(".route(\"/api/{0}\", delete(delete_{0}))", model.name).as_str());
    }
    code.push_str(
        r#".merge(axum_extra::routing::SpaRouter::new("/assets", "../dist"))
    .layer(Extension(pool))
}"#,
    );

    code
}

fn create_main_fn() {
    let code = r#"
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

    "#;
}

fn create_or_get_src_dir(user_id: &str) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let data_path = current_dir.join("../").join(user_id).join("src");
    fs::create_dir_all(&data_path).unwrap();
    data_path
}

fn create_or_get_project_dir(user_id: &str) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let data_path = current_dir.join("../").join(user_id);
    fs::create_dir_all(&data_path).unwrap();
    data_path
}

fn write_code(code: &str, user_id: &str, file_name: &str) {
    let data_path = create_or_get_src_dir(user_id).join(&file_name);
    let mut main_rs = fs::File::create(data_path).unwrap();
    main_rs.write_all(code.as_bytes()).unwrap();
}

fn write_model(user_id: &str, models: InternalModels) {
    let app_fn = create_app_fn(models);

    write_code(&app_fn, user_id, "main.rs");
}

fn get_cargo_toml(user_id: &str) -> String {
    let cargo_toml = include_str!("../templates/Cargo.toml");
    cargo_toml.replace("name = \"\"", format!("name = \"{}\"", user_id).as_str())
}

pub fn write_all(user_id: &str, models: InternalModels) {
    let cargo_toml = get_cargo_toml(user_id);
    let data_path = create_or_get_project_dir(user_id).join("Cargo.toml");
    let mut main_rs = fs::File::create(data_path).unwrap();
    main_rs.write_all(cargo_toml.as_bytes()).unwrap();

    write_model(user_id, models);
}

#[cfg(test)]
mod tests {

    use super::*;

    use super::super::InternalModel;

    #[test]
    fn test_write_all() {
        let models = vec![InternalModel {
            name: "Order".to_string(),
        }];
        write_all("user_id", models);
    }

    // #[test]
    // fn test_write_model() {
    //     let models = vec![InternalModel{name: "Order".to_string()}];
    //     write_model(models);
    // }

    // fn write_code(code: &str) {
    //     super::write_code(code, "user_id");
    // }

    // #[test]
    // fn test_create_app_fn() {
    //     let models = vec![InternalModel{name: "Order".to_string()}];
    //     eprintln!("order_with_id = {:#?}", create_app_fn(models));
    //     assert_eq!(5, 4);
    // }

    // #[test]
    // fn test_write_code() {
    //     write_code("sdflkj");
    //     assert_eq!(3, 4);
    // }

    // #[test]
    // fn convert_simple_id() {
    //     let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}});
    //     convert(&order_with_id);
    //     println!("{:?}", order_with_id);
    //     assert_eq!(5, 4);
    // }
}
