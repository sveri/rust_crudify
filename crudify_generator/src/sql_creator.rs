use indexmap::IndexMap;
use phf::phf_map;

use crate::{json_converter::RustDataType, InternalModel, errors::SqlConverterError};

pub fn create_get_all_entities(model: &InternalModel) -> String {
    let mut sql: String = "SELECT ".to_string();

    let fields: String = match &model.properties {
        None => "*".to_string(),
        Some(properties) => properties.keys().map(|p| p.as_ref()).collect::<Vec<_>>().join(", "),
    };

    sql.push_str(&fields);
    sql.push_str(&format!(" FROM public.{}", &model.name.to_lowercase()));

    sql
}

pub fn create_create_entity(model: &InternalModel) -> String {
    let mut sql: String = "INSERT INTO ".to_string();
    sql.push_str(&format!("public.{} ", &model.name.to_lowercase()));

    let fields = format!(
        "({})",
        model
            .properties
            .as_ref()
            .map(|properties| properties.keys().map(|p| p.as_ref()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    );

    sql.push_str(&fields);
    sql.push_str(" VALUES ");

    let values: String = match &model.properties {
        None => "".to_string(),
        Some(properties) => {
            format!(
                "({})",
                (1..properties.keys().len() + 1)
                    .map(|idx| format!("${}", idx))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    };

    sql.push_str(&values);

    sql
}

pub fn create_update_entity(model: &InternalModel) -> String {
    let fields: String = match &model.properties {
        None => "".to_string(),
        Some(properties) => properties
            .keys()
            .enumerate()
            .into_iter()
            .map(|(idx, k)| format!("{} = ${}", k, idx + 1))
            .collect::<Vec<_>>()
            .join(", "),
    };

    format!("UPDATE public.{} SET {} WHERE id = $1", model.name.to_lowercase(), fields)
}

pub fn create_delete_entity(model: &InternalModel) -> String {
    format!("DELETE FROM public.{} WHERE id = ?", model.name.to_lowercase())
}

pub fn create_create_table(model: &InternalModel) -> String {
    let fields: String = match &model.properties {
        None => "".to_string(),
        Some(properties) => properties
            .into_iter()
            .map(|(key, value)| {
                format!("{} {}", key, get_matching_sql_datatype(&value).unwrap())
            })
            .collect::<Vec<_>>()
            .join(", "),
    };

    format!("CREATE TABLE IF NOT EXISTS public.{} ({});", model.name.to_lowercase(), fields)
}

fn get_matching_sql_datatype(data_type: &RustDataType) -> Result<&'static str, SqlConverterError<'static>> {
    match data_type {
        RustDataType::U8 => Ok("smallint"),
        RustDataType::I32 => Ok("integer"),
        RustDataType::I64 => Ok("bigint"),
        RustDataType::F32 => Ok("real"),
        RustDataType::F64 => Ok("doubl precision"),
        RustDataType::String => Ok("text"),
        RustDataType::Bool => Ok("boolean"),
        RustDataType::Date => Ok("date"),
        RustDataType::DateTime => Ok("datetime"),
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};

    use crate::json_converter::RustDataType;

    use super::*;

    impl InternalModel {
        fn new(name: String) -> InternalModel {
            InternalModel { name, properties: None }
        }

        fn new_with_props(name: String, props: IndexMap<String, RustDataType>) -> InternalModel {
            InternalModel {
                name,
                properties: Some(props),
            }
        }
    }

    #[test]
    fn test_create_table() {
        let props = indexmap! {"id".to_string() => RustDataType::I64, "name".to_string() => RustDataType::String};
        let expected = "CREATE TABLE IF NOT EXISTS public.order (id bigint, name text);";
        assert_eq!(
            expected,
            create_create_table(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_delete_entity() {
        let props = indexmap! {"id".to_string() => RustDataType::I64, "name".to_string() => RustDataType::String};
        let expected = "DELETE FROM public.order WHERE id = ?";
        assert_eq!(
            expected,
            create_delete_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_update_entity_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => RustDataType::I64, "name".to_string() => RustDataType::String};
        let expected = "UPDATE public.order SET id = $1, name = $2 WHERE id = $1";
        assert_eq!(
            expected,
            create_update_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_update_entity_with_one_properties() {
        let props = indexmap! {"id".to_string() => RustDataType::I64};
        let expected = "UPDATE public.order SET id = $1 WHERE id = $1";
        assert_eq!(
            expected,
            create_update_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_create_entity_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => RustDataType::I64, "name".to_string() => RustDataType::String};
        let expected = "INSERT INTO public.order (id, name) VALUES ($1, $2)";
        assert_eq!(
            expected,
            create_create_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_create_entity_with_one_property() {
        let props = indexmap! {"id".to_string() => RustDataType::I64};
        let expected = "INSERT INTO public.order (id) VALUES ($1)";
        assert_eq!(
            expected,
            create_create_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_get_entities_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => RustDataType::I64, "name".to_string() => RustDataType::String};
        let expected = "SELECT id, name FROM public.order";
        assert_eq!(
            expected,
            create_get_all_entities(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_get_entities_with_property() {
        let props = indexmap! {"id".to_string() => RustDataType::I64};
        let expected = "SELECT id FROM public.order";
        assert_eq!(
            expected,
            create_get_all_entities(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_get_entities_without_properties() {
        let expected = "SELECT * FROM public.order";
        assert_eq!(expected, create_get_all_entities(&InternalModel::new("Order".to_string())));
    }
}
