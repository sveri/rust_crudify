// json -> rust code -> file system

use serde_json::{json, Value};

pub struct InternalModel {
    pub name: String
}

pub type InternalModels = Vec<InternalModel>;


fn convert_to_internal_model(j: &Value) -> InternalModels {
    let mut internal_models = Vec::new();
    for(key, value) in j.as_object().unwrap() {
        println!("{:?} - {:?}", key, value);
        internal_models.push(InternalModel{name: key.to_string()})
    }

    internal_models

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_simple_id() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}, "OrderTwo": {}});
        let models = convert_to_internal_model(&order_with_id);
        println!("{:?}", order_with_id);
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!("OrderTwo", models.get(1).unwrap().name);
    }
}