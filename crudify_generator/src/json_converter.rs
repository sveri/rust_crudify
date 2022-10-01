// json -> rust code -> file system

use indexmap::IndexMap;
use serde_json::{Value};

use crate::{InternalModels, InternalModel};


pub fn convert_to_internal_model(j: &Value) -> InternalModels {
    let mut internal_models = Vec::new();
    for(key, value) in j.as_object().unwrap() {
        if value.is_object() {
            let properties = parse_properties(&value);
    
            internal_models.push(InternalModel{name: key.to_string(), properties: Some(properties)})
        }
    }

    internal_models
}


fn parse_properties(value: &Value) -> IndexMap<String, String> {
    let mut property_map: IndexMap<String, String> = IndexMap::new();
    let o = value.as_object().unwrap();
    if o.contains_key("properties") {
        let properties = o.get("properties").unwrap();

        for(property_key, property_value) in properties.as_object().unwrap() {
            if property_key == "id" {
                property_map.insert(property_key.to_string(), property_key.to_string());
            }
        }
    }

    property_map
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn with_id_property() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}});
        let models = convert_to_internal_model(&order_with_id);
        println!("{:?}", order_with_id);
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!("id".to_string(), &models.get(0).unwrap().properties.unwrap().get("id").unwrap().to_string());
    }

    #[test]
    fn without_properties() {
        let two_order_objects = json!({"Order": {}, "OrderTwo": {}});
        let models = convert_to_internal_model(&two_order_objects);
        println!("{:?}", two_order_objects);
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!("OrderTwo", models.get(1).unwrap().name);
    }
}