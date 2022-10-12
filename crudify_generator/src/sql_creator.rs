use indexmap::{indexmap, IndexMap};

use crate::InternalModel;

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
        model.properties.as_ref().map(|properties| properties
            .keys()
            .map(|p| p.as_ref())
            .collect::<Vec<_>>()
            .join(", ")).unwrap_or_default()
            // model.properties.as_ref().map_or_else(Default::default, |properties| properties
            //     .keys()
            //     .map(|p| p.as_ref())
            //     .collect::<Vec<_>>()
            //     .join(", "))
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
        Some(properties) => {
            properties.keys().enumerate().into_iter().map(|(idx, k)| format!("{} = ${}", k, idx + 1)).collect::<Vec<_>>().join(", ")
        }
    };

    format!("UPDATE public.{} SET {} WHERE id = $1", model.name.to_lowercase(), fields)
}

pub fn create_delete_entity(model: &InternalModel) -> String {
    format!("DELETE FROM public.{} WHERE id = ?", model.name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};

    use super::*;

    impl InternalModel {
        fn new(name: String) -> InternalModel {
            InternalModel { name, properties: None }
        }

        fn new_with_props(name: String, props: IndexMap<String, String>) -> InternalModel {
            InternalModel {
                name,
                properties: Some(props),
            }
        }
    }

    #[test]
    fn test_delete_entity() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "DELETE FROM public.order WHERE id = ?";
        assert_eq!(
            expected,
            create_delete_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_update_entity_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "UPDATE public.order SET id = $1, name = $2 WHERE id = $1";
        assert_eq!(
            expected,
            create_update_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_update_entity_with_one_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string()};
        let expected = "UPDATE public.order SET id = $1 WHERE id = $1";
        assert_eq!(
            expected,
            create_update_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_create_entity_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "INSERT INTO public.order (id, name) VALUES ($1, $2)";
        assert_eq!(
            expected,
            create_create_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_create_entity_with_one_property() {
        let props = indexmap! {"id".to_string() => "i64".to_string()};
        let expected = "INSERT INTO public.order (id) VALUES ($1)";
        assert_eq!(
            expected,
            create_create_entity(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_get_entities_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "SELECT id, name FROM public.order";
        assert_eq!(
            expected,
            create_get_all_entities(&InternalModel::new_with_props("Order".to_string(), props))
        );
    }

    #[test]
    fn test_get_entities_with_property() {
        let props = indexmap! {"id".to_string() => "i64".to_string()};
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
