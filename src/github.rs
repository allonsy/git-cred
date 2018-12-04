
use serde_json::value::Value;
use reqwest::get;

pub fn get_key(user: &str, key_id: Option<String>) -> Option<String> {
    let url = format!("https://api.github.com/users/{}/gpg_keys", user);
    let response = get(&url);
    if response.is_err() {
        return None;
    }

    let mut unwrapped_response = response.unwrap();
    let key_array: reqwest::Result<Value> = unwrapped_response.json();
    if key_array.is_err() {
        return None;
    }

    let key_array_res = key_array.unwrap();

    let keys = match key_array_res {
        Value::Array(keys) => {
            keys
        },
        _ => {
            return None;
        }
    };

    for key in keys {
        match key {
            Value::Object(omap) => {
                if key_id.is_none() {
                    if omap.contains_key("raw_key") {
                        let mut key_string = omap.get("raw_key").as_ref().unwrap().to_string();
                        key_string = key_string.replace("\"", "");
                        key_string = key_string.replace("\\r\\n", "\n");
                        return Some(key_string);
                    }
                } else {
                    if omap.contains_key("key_id") {
                        let omap_key_id = omap.get("key_id").unwrap();
                        if omap_key_id.is_string() {
                            if key_id.as_ref().unwrap().ends_with(omap_key_id.as_str().unwrap()) {
                                if omap.contains_key("raw_key") {
                                    let mut key_string = omap.get("raw_key").as_ref().unwrap().to_string();
                                    key_string = key_string.replace("\"", "");
                                    key_string = key_string.replace("\\r\\n", "\n");
                                    return Some(key_string);
                                }
                            }
                        }
                    }
                }
            },
            _ => {
                return None;
            }
        }
    }
    return None;
}