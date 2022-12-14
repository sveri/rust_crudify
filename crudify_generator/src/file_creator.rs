use std::{fs, io::Write, path::PathBuf};

use super::InternalModels;

use super::main_file_creator::write_main_file;

pub fn create_or_get_project_dir(user_id: &str) -> Result<PathBuf, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    let data_path = current_dir.join("../").join(user_id);
    fs::create_dir_all(&data_path)?;
    Ok(data_path)
}

fn write_cargo_toml(user_id: &str) -> Result<(), std::io::Error> {
    let cargo_toml = include_str!("../templates/Cargo.toml");
    let cargo_toml = cargo_toml.replace("name = \"\"", format!("name = \"{}\"", user_id).as_str());
    let data_path = create_or_get_project_dir(user_id)?.join("Cargo.toml");
    let mut main_rs = fs::File::create(data_path)?;
    main_rs.write_all(cargo_toml.as_bytes())?;
    Ok(())
}

pub fn write_all(user_id: &str, models: &InternalModels) -> Result<(), std::io::Error> {
    write_cargo_toml(user_id)?;
    write_main_file(user_id, models)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    // impl InternalModel {
    //     fn new(name: String) -> InternalModel {
    //         InternalModel { name, properties: None }
    //     }
    // }

    // use super::*;

    // use super::super::InternalModel;

    // #[test]
    // fn test_write_all() {
    //     // let models = vec![InternalModel { name: "Order".to_string() }];
    //     let models = vec![InternalModel::new("Order".to_string())];
    //     write_all("user_id", &models);
    // }

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
