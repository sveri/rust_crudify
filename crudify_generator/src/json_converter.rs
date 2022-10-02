// json -> rust code -> file system

use std::{error::Error, fmt::Display};

use indexmap::IndexMap;
use log::{warn, info, error};
use phf::phf_map;
use serde_json::{Value};

use crate::{InternalModels, InternalModel};

// from https://www.reddit.com/r/rust/comments/gj8inf/comment/fqlmknt/
#[derive(Debug)]
pub enum JsonConverterError {
    AsObjectError,
}

impl Error for JsonConverterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::AsObjectError => None
        }
    }
}

impl Display for JsonConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::AsObjectError => {
                write!(f, "Could not convert value to object")
            }
        }
    }
}

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_FORMAT_TO_RUST_DATATYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "int64" => "i64",
    "int32" => "i32",
    "date" => "chrono::Date",
    "date-time" => "chrono::DateTime",
    "password" => "String",
    "byte" => "u8",
    "boolean" => "bool",
    "float" => "f32",
    "double" => "f64"
};

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_TO_RUST_DATATYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "integer" => "i64",
    "string" => "String",
    "boolean" => "bool",
    "number" => "f64"
};


pub fn convert_to_internal_model(j: &Value) -> Result<InternalModels, JsonConverterError> {
    let mut internal_models = Vec::new();
    for(key, value) in j.as_object().unwrap() {
        if value.is_object() {
            let properties = parse_properties(&value)?;
    
            internal_models.push(InternalModel{name: key.to_string(), properties: Some(properties)})
        }
    }

    Ok(internal_models)
}


fn parse_properties(value: &Value) -> Result<IndexMap<String, String>, JsonConverterError> {
    let mut property_map: IndexMap<String, String> = IndexMap::new();
    // let o = value.as_object().ok_or(format!("parameter value is not an object: {:?}", value))?;
    let o = value.as_object().ok_or(JsonConverterError::AsObjectError)?;
    if let Some(properties) = o.get("properties") {

        for(property_key, property_value) in properties.as_object().ok_or(JsonConverterError::AsObjectError)? {
            if property_key == "id" && property_value.is_object() {
                
                let property_type = property_value.as_object().unwrap();
                let mut data_type = "String";
                if property_type.contains_key("format") {
                    data_type = DATATYPE_FORMAT_TO_RUST_DATATYPE.get(property_type.get("format").unwrap().as_str().unwrap()).unwrap();
                }

                property_map.insert(property_key.to_string(), data_type.to_string());
            } else {
                warn!("The id value must be an object, but was {:?}", property_value);
            }

        }
    }

    Ok(property_map)
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;


    #[test]
    fn non_object_value_must_err() {
        let order_with_id = json!({"Order": {"type": "object", "properties": []}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        let err = models.unwrap_err();
        assert_eq!("sdlkfj", err.to_string());
        // assert_eq!("Order", models.unwrap().get(0).unwrap().name);
        // assert_eq!("i64".to_string(), models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string());
    }

    // #[test]
    // fn with_id_property() {
    //     let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}});
    //     let models = convert_to_internal_model(&order_with_id);
    //     assert_eq!("Order", models.get(0).unwrap().name);
    //     assert_eq!("i64".to_string(), models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string());
    // }

    // #[test]
    // fn with_wrong_id_property() {
    //     init();
    //     let order_with_id = json!({"Order": {"type": "object", "properties": {"id": "foobar"}}});
    //     let models = convert_to_internal_model(&order_with_id);
    //     assert_eq!("Order", models.get(0).unwrap().name);
    // }

    // #[test]
    // fn without_properties() {
    //     let two_order_objects = json!({"Order": {}, "OrderTwo": {}});
    //     let models = convert_to_internal_model(&two_order_objects);
    //     assert_eq!("Order", models.get(0).unwrap().name);
    //     assert_eq!("OrderTwo", models.get(1).unwrap().name);
    // }
    
    fn init() {
        let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();
    }
}