use std::{fs, path::PathBuf, time::Duration};

use serde_json::Value;

const API_BASE: &str = "https://api.changes.tg";

#[derive(Debug, Clone)]
pub struct GiftModel {
    pub name: String,
    pub rarity: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct GiftDetails {
    pub name: String,
    pub id: Option<String>,
    pub models: Vec<GiftModel>,
}

#[derive(Clone)]
pub struct GiftChangesClient {
    client: reqwest::blocking::Client,
}

impl GiftChangesClient {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(concat!("GiftWallpaper/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| format!("failed to create HTTP client: {e}"))?;

        Ok(Self { client })
    }

    pub fn list_gifts(&self) -> Result<Vec<String>, String> {
        let value = self.get_json(&format!("{API_BASE}/gifts"))?;
        let mut gifts = parse_gift_names(&value);

        gifts.sort_by_key(|name| name.to_lowercase());
        gifts.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

        if gifts.is_empty() {
            return Err("the API returned no gift names".to_owned());
        }

        Ok(gifts)
    }

    pub fn gift_details(&self, gift: &str) -> Result<GiftDetails, String> {
        let gift = urlencoding::encode(gift);
        let value = self.get_json(&format!("{API_BASE}/gift/{gift}"))?;
        parse_gift_details(&value)
    }

    pub fn model_png(&self, gift: &str, model: &str, size: u32) -> Result<Vec<u8>, String> {
        let cache_path = cache_file(gift, model, size);

        if let Ok(bytes) = fs::read(&cache_path)
            && !bytes.is_empty()
        {
            return Ok(bytes);
        }

        let gift = urlencoding::encode(gift);
        let model = urlencoding::encode(model);
        let url = format!("{API_BASE}/model/{gift}/{model}.png?size={size}");

        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| format!("image request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("image request returned an error: {e}"))?;

        let bytes = response
            .bytes()
            .map_err(|e| format!("failed to read image: {e}"))?
            .to_vec();

        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cache_path, &bytes);

        Ok(bytes)
    }

    fn get_json(&self, url: &str) -> Result<Value, String> {
        self.client
            .get(url)
            .send()
            .map_err(|e| format!("API request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("API returned an error: {e}"))?
            .json::<Value>()
            .map_err(|e| format!("invalid JSON from API: {e}"))
    }
}

fn parse_gift_names(value: &Value) -> Vec<String> {
    if let Some(array) = value.as_array() {
        return array.iter().filter_map(value_to_name).collect();
    }

    if let Some(object) = value.as_object() {
        for key in ["gifts", "names", "data"] {
            if let Some(array) = object.get(key).and_then(Value::as_array) {
                let names: Vec<String> = array.iter().filter_map(value_to_name).collect();
                if !names.is_empty() {
                    return names;
                }
            }
        }

        // Some APIs use {"Gift Name": "..."} or {"Gift Name": id}.
        let keys: Vec<String> = object
            .keys()
            .filter(|key| !key.trim().is_empty())
            .cloned()
            .collect();

        if !keys.is_empty() {
            return keys;
        }
    }

    Vec::new()
}

fn value_to_name(value: &Value) -> Option<String> {
    if let Some(name) = value.as_str() {
        let name = name.trim();
        return (!name.is_empty()).then(|| name.to_owned());
    }

    value
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_gift_details(value: &Value) -> Result<GiftDetails, String> {
    let nested_gift = value.get("gift");

    let name = nested_gift
        .and_then(|gift| gift.get("name"))
        .and_then(Value::as_str)
        .or_else(|| value.get("name").and_then(Value::as_str))
        .unwrap_or("Unknown Gift")
        .to_owned();

    let id = nested_gift
        .and_then(|gift| gift.get("id"))
        .or_else(|| value.get("id"))
        .and_then(value_to_string);

    let models_value = value
        .get("models")
        .or_else(|| value.get("data").and_then(|data| data.get("models")));

    let mut models = models_value
        .and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(|model| {
                    let name = model
                        .get("name")
                        .and_then(Value::as_str)
                        .or_else(|| model.as_str())?
                        .trim()
                        .to_owned();

                    if name.is_empty() {
                        return None;
                    }

                    Some(GiftModel {
                        name,
                        rarity: model.get("rarity").and_then(parse_number),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    models.sort_by_key(|model| model.name.to_lowercase());

    if models.is_empty() {
        return Err(format!(
            "gift '{name}' was loaded, but no models were found in the response"
        ));
    }

    Ok(GiftDetails { name, id, models })
}

fn parse_number(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }

    let text = value.as_str()?.trim().trim_end_matches('%');
    text.parse::<f64>().ok()
}

fn value_to_string(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    if let Some(number) = value.as_u64() {
        return Some(number.to_string());
    }
    if let Some(number) = value.as_i64() {
        return Some(number.to_string());
    }
    None
}

fn cache_file(gift: &str, model: &str, size: u32) -> PathBuf {
    cache_dir().join(format!(
        "{}__{}__{}.png",
        file_safe(gift),
        file_safe(model),
        size
    ))
}

fn cache_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(base) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(base).join("GiftWallpaper").join("cache");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(base) = std::env::var_os("XDG_CACHE_HOME") {
            return PathBuf::from(base).join("giftwallpaper");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".cache").join("giftwallpaper");
        }
    }

    std::env::temp_dir().join("giftwallpaper-cache")
}

fn file_safe(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_gift_list() {
        let value = serde_json::json!(["Scared Cat", "Sakura Flower"]);
        assert_eq!(
            parse_gift_names(&value),
            vec!["Scared Cat".to_owned(), "Sakura Flower".to_owned()]
        );
    }

    #[test]
    fn parses_gift_details() {
        let value = serde_json::json!({
            "gift": {"name": "Scared Cat", "id": "123"},
            "models": [
                {"name": "Black Cat", "rarity": 0.5},
                {"name": "White Cat", "rarity": "1.2%"}
            ]
        });

        let details = parse_gift_details(&value).unwrap();
        assert_eq!(details.name, "Scared Cat");
        assert_eq!(details.id.as_deref(), Some("123"));
        assert_eq!(details.models.len(), 2);
    }
}
