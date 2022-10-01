use indexmap::IndexMap;
use serde_json::{Value};

mod file_creator;
mod main_file_creator;
mod json_converter;


pub struct InternalModel {
    pub name: String,
    pub properties: Option<IndexMap<String, String>>
}

impl InternalModel {
    fn new(name: String) -> InternalModel {
        InternalModel { name: name, properties: None }
    }
}

pub type InternalModels = Vec<InternalModel>;

pub fn generate(user_id: &str, input_objects: &Value) {
    let models = json_converter::convert_to_internal_model(&input_objects);
    file_creator::write_all(user_id, &models);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_works() {
        let order_with_id = json!({"Order": {}, "OrderTwo": {}});
        generate("user_id", &order_with_id);
    }
}
