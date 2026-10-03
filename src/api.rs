use std::{fs, path::PathBuf, time::Duration};

use serde_json::Value;

const API_BASE: &str = "https://api.changes.tg";

#[derive(Debug, Clone)]
pub struct GiftModel {
    pub name: String,
    pub rarity: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GiftBackdropColors {
    pub center: [u8; 3],
    pub edge: [u8; 3],
    pub symbol: [u8; 3],
    pub text: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct GiftBackdrop {
    pub name: String,
    pub rarity: Option<f64>,
    pub colors: Option<GiftBackdropColors>,
}

#[derive(Debug, Clone)]
pub struct GiftDetails {
    pub name: String,
    pub id: Option<String>,
    pub models: Vec<GiftModel>,
    pub backdrops: Vec<GiftBackdrop>,
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

    pub fn backdrop_info(&self, gift: &str, backdrop: &str) -> Result<GiftBackdrop, String> {
        let backdrop_name = backdrop.to_owned();
        let gift = urlencoding::encode(gift);
        let backdrop = urlencoding::encode(backdrop);
        let value = self.get_json(&format!("{API_BASE}/backdrop/{gift}/{backdrop}/info"))?;

        if let Some(info) = parse_backdrop(&value)
            .or_else(|| value.get("backdrop").and_then(parse_backdrop))
            .or_else(|| value.get("data").and_then(parse_backdrop))
        {
            return Ok(info);
        }

        let color_source = value
            .get("backdrop")
            .or_else(|| value.get("data"))
            .unwrap_or(&value);

        let colors = parse_backdrop_colors(color_source)
            .ok_or_else(|| "the API returned backdrop info without readable colors".to_owned())?;

        let rarity = color_source
            .get("rarity")
            .and_then(parse_number)
            .or_else(|| {
                color_source
                    .get("rarity_per_mille")
                    .and_then(parse_number)
                    .map(|value| value / 10.0)
            });

        Ok(GiftBackdrop {
            name: backdrop_name,
            rarity,
            colors: Some(colors),
        })
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
        .or_else(|| value.get("data").and_then(|data| data.get("models")))
        .or_else(|| nested_gift.and_then(|gift| gift.get("models")));

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
                        rarity: model.get("rarity").and_then(parse_number).or_else(|| {
                            model
                                .get("rarity_per_mille")
                                .and_then(parse_number)
                                .map(|value| value / 10.0)
                        }),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    models.sort_by_key(|model| model.name.to_lowercase());

    let backdrops_value = value
        .get("backdrops")
        .or_else(|| value.get("data").and_then(|data| data.get("backdrops")))
        .or_else(|| nested_gift.and_then(|gift| gift.get("backdrops")));

    let mut backdrops = backdrops_value.map(parse_backdrops).unwrap_or_default();

    backdrops.sort_by_key(|backdrop| backdrop.name.to_lowercase());

    if models.is_empty() {
        return Err(format!(
            "gift '{name}' was loaded, but no models were found in the response"
        ));
    }

    Ok(GiftDetails {
        name,
        id,
        models,
        backdrops,
    })
}

fn parse_backdrops(value: &Value) -> Vec<GiftBackdrop> {
    if let Some(array) = value.as_array() {
        return array.iter().filter_map(parse_backdrop).collect();
    }

    let Some(object) = value.as_object() else {
        return Vec::new();
    };

    if object.contains_key("name") {
        return parse_backdrop(value).into_iter().collect();
    }

    object
        .iter()
        .filter_map(|(name, entry)| {
            if let Some(mut entry_object) = entry.as_object().cloned() {
                entry_object
                    .entry("name".to_owned())
                    .or_insert_with(|| Value::String(name.clone()));
                return parse_backdrop(&Value::Object(entry_object));
            }

            let name = name.trim();
            if name.is_empty() {
                return None;
            }

            Some(GiftBackdrop {
                name: name.to_owned(),
                rarity: parse_number(entry),
                colors: None,
            })
        })
        .collect()
}

fn parse_backdrop(value: &Value) -> Option<GiftBackdrop> {
    if let Some(name) = value.as_str() {
        let name = name.trim();
        return (!name.is_empty()).then(|| GiftBackdrop {
            name: name.to_owned(),
            rarity: None,
            colors: None,
        });
    }

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())?
        .to_owned();

    let rarity = value.get("rarity").and_then(parse_number).or_else(|| {
        value
            .get("rarity_per_mille")
            .and_then(parse_number)
            .map(|value| value / 10.0)
    });

    let colors = parse_backdrop_colors(value);

    Some(GiftBackdrop {
        name,
        rarity,
        colors,
    })
}

fn parse_backdrop_colors(value: &Value) -> Option<GiftBackdropColors> {
    let colors = value
        .get("colors")
        .or_else(|| value.get("colour"))
        .or_else(|| value.get("palette"))
        .unwrap_or(value);

    let center = color_field(colors, &["center_color", "centerColor", "center"])?;
    let edge = color_field(colors, &["edge_color", "edgeColor", "edge"])?;
    let symbol = color_field(
        colors,
        &[
            "symbol_color",
            "symbolColor",
            "pattern_color",
            "patternColor",
            "symbol",
            "pattern",
        ],
    )
    .unwrap_or(center);
    let text = color_field(colors, &["text_color", "textColor", "text"]).unwrap_or(symbol);

    Some(GiftBackdropColors {
        center,
        edge,
        symbol,
        text,
    })
}

fn color_field(value: &Value, keys: &[&str]) -> Option<[u8; 3]> {
    for key in keys {
        if let Some(color) = value.get(*key).and_then(parse_rgb) {
            return Some(color);
        }
    }

    None
}

fn parse_rgb(value: &Value) -> Option<[u8; 3]> {
    if let Some(number) = value.as_u64() {
        return rgb_from_u64(number);
    }

    if let Some(number) = value.as_i64()
        && number >= 0
    {
        return rgb_from_u64(number as u64);
    }

    if let Some(text) = value.as_str() {
        let text = text.trim();
        let hex = text
            .strip_prefix('#')
            .or_else(|| text.strip_prefix("0x"))
            .or_else(|| text.strip_prefix("0X"));

        if let Some(hex) = hex
            && let Ok(number) = u64::from_str_radix(hex, 16)
        {
            return rgb_from_u64(number);
        }

        if text.len() == 6
            && text.chars().all(|ch| ch.is_ascii_hexdigit())
            && let Ok(number) = u64::from_str_radix(text, 16)
        {
            return rgb_from_u64(number);
        }

        if let Ok(number) = text.parse::<u64>() {
            return rgb_from_u64(number);
        }
    }

    if let Some(array) = value.as_array()
        && array.len() >= 3
    {
        let red = array[0].as_u64()?.min(255) as u8;
        let green = array[1].as_u64()?.min(255) as u8;
        let blue = array[2].as_u64()?.min(255) as u8;
        return Some([red, green, blue]);
    }

    let object = value.as_object()?;
    let red = object.get("r")?.as_u64()?.min(255) as u8;
    let green = object.get("g")?.as_u64()?.min(255) as u8;
    let blue = object.get("b")?.as_u64()?.min(255) as u8;
    Some([red, green, blue])
}

fn rgb_from_u64(number: u64) -> Option<[u8; 3]> {
    if number > 0xFF_FF_FF {
        return None;
    }

    Some([
        ((number >> 16) & 0xFF) as u8,
        ((number >> 8) & 0xFF) as u8,
        (number & 0xFF) as u8,
    ])
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
    fn parses_gift_details_with_backdrop_colors() {
        let value = serde_json::json!({
            "gift": {"name": "Scared Cat", "id": "123"},
            "models": [
                {"name": "Black Cat", "rarity": 0.5},
                {"name": "White Cat", "rarity": "1.2%"}
            ],
            "backdrops": [
                {
                    "name": "Aquamarine",
                    "rarity_per_mille": 10,
                    "colors": {
                        "center_color": 0x55AAFF,
                        "edge_color": 0x123456,
                        "symbol_color": 0x88CCFF,
                        "text_color": 0xFFFFFF
                    }
                }
            ]
        });

        let details = parse_gift_details(&value).unwrap();
        assert_eq!(details.name, "Scared Cat");
        assert_eq!(details.id.as_deref(), Some("123"));
        assert_eq!(details.models.len(), 2);
        assert_eq!(details.backdrops.len(), 1);
        assert_eq!(details.backdrops[0].name, "Aquamarine");
        assert_eq!(
            details.backdrops[0].colors.unwrap().center,
            [0x55, 0xAA, 0xFF]
        );
    }

    #[test]
    fn parses_hex_backdrop_colors() {
        let value = serde_json::json!({
            "name": "Amber",
            "colors": {
                "centerColor": "#ffb347",
                "edgeColor": "8a4b08",
                "symbolColor": "0xFFD27F",
                "textColor": 16777215
            }
        });

        let backdrop = parse_backdrop(&value).unwrap();
        let colors = backdrop.colors.unwrap();

        assert_eq!(colors.center, [0xFF, 0xB3, 0x47]);
        assert_eq!(colors.edge, [0x8A, 0x4B, 0x08]);
        assert_eq!(colors.symbol, [0xFF, 0xD2, 0x7F]);
        assert_eq!(colors.text, [0xFF, 0xFF, 0xFF]);
    }
}
