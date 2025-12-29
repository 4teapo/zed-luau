use zed_extension_api::serde_json::{Map, Value, map::Entry};

pub fn get_or_insert_object<'a>(
    map: &'a mut Map<String, Value>,
    key: &str,
) -> &'a mut Map<String, Value> {
    match map.entry(key.to_string()) {
        Entry::Vacant(e) => {
            if let Value::Object(o) = e.insert(Value::Object(Map::new())) {
                o
            } else {
                unreachable!()
            }
        }
        Entry::Occupied(mut e) => {
            if !e.get().is_object() {
                e.insert(Value::Object(Map::new()));
            }
            if let Value::Object(o) = e.into_mut() {
                o
            } else {
                unreachable!()
            }
        }
    }
}

pub fn get_or_insert_array<'a>(map: &'a mut Map<String, Value>, key: &str) -> &'a mut Vec<Value> {
    match map.entry(key.to_string()) {
        Entry::Vacant(e) => {
            if let Value::Array(o) = e.insert(Value::Array(Vec::new())) {
                o
            } else {
                unreachable!()
            }
        }
        Entry::Occupied(mut e) => {
            if !e.get().is_array() {
                e.insert(Value::Array(Vec::new()));
            }
            if let Value::Array(o) = e.into_mut() {
                o
            } else {
                unreachable!()
            }
        }
    }
}
