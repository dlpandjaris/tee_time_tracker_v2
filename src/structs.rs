use serde::{Deserialize, Serialize, Deserializer};
use std::sync::Arc;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerboseCourseId {
    pub id: i64,
    pub url: String,
    pub alias: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CourseId {
    Number(i64),
    String(String),
    Verbose(VerboseCourseId)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GolfCourse {
    pub id: CourseId,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct Coords {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub courses: Arc<Vec<GolfCourse>>,
}

#[derive(Debug, Serialize)]
pub struct TeeTime {
    pub course: String,
    pub tee_time: DateTime<Utc>,
    pub price: f64,
    pub players: u32,
    pub holes: Option<u32>,
    pub lat: f64,
    pub lon: f64,
    pub book_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GolfBackResponse {
    #[serde(default)]
    pub data: Vec<GolfBackTeeTime>,
}

#[derive(Debug, Deserialize)]
pub struct GolfBackPrimaryPrices {
    pub basePrice: f64,
    pub holes: i64,
    pub price: f64,
}

#[derive(Debug, Deserialize)]
pub struct GolfBackTeeTime {
    pub courseId: String,
    pub courseName: String,
    pub dateTime: String,
    pub has9Holes: bool,
    pub hasDailyGimmeV2: bool,
    pub hasDeal: bool,
    pub holes: Vec<u32>,
    pub id: String,
    pub isAvailable: bool,
    pub localDateTime: String,
    pub location: Option<String>,
    pub lockExpiration: Option<String>,
    pub playersDisplay: String,
    pub playersMax: u32,
    pub playersMin: u32,
    pub primaryPrices: Vec<GolfBackPrimaryPrices>,
    pub rates: Vec<GolfBackRate>,
}

#[derive(Debug, Deserialize)]
pub struct GolfBackRate {
    pub basePrice: f64, // 34.0,
    pub description: String, // "All you can play, up to 18 holes",
    pub feeDisplay: f64, // 0.0,
    pub hasCartIncluded: bool, // true,
    pub holes: u32, // 18,
    pub isDailyGimmeV2: bool, // false,
    pub isDeal: bool, // false,
    pub isGimme: bool, // false,
    pub isPrimary: bool, // true,
    pub name: String, // "Twilight",
    pub price: f64, // 34.0,
    pub ratePlanId: String, // "0bc2bf83-2bf5-4ba1-be8b-a06691bf761a",
    pub usePrimaryAfterSelection: bool, // false
}

#[derive(Debug, Deserialize)]
pub struct ForeUpTeeTime {
    pub time: String,
    pub green_fee: f64,
    pub cart_fee: f64,
    pub available_spots: u32,

    #[serde(deserialize_with = "deserialize_holes")]
    pub holes: u32,
}


fn deserialize_holes<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a serde_json::Value first
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => {
            n.as_u64()
                .map(|v| v as u32)
                .ok_or_else(|| serde::de::Error::custom("Invalid number for holes"))
        }
        serde_json::Value::String(s) => {
            // If string contains "18", return 18, otherwise 9
            if s.contains("18") {
                Ok(18)
            } else {
                Ok(9)
            }
        }
        other => Err(serde::de::Error::custom(format!(
            "Unexpected type for holes: {:?}",
            other
        ))),
    }
}





// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GolfBackResponse {
//     pub data: Vec<GolfBackTeeTime>,
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GolfBackPrimaryPrices {
//     pub base_price: f64,
//     pub holes: i64,
//     pub price: f64,
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GolfBackTeeTime {
//     pub course_id: String,
//     pub course_name: String,
//     pub date_time: String,
//     pub has9_holes: bool,
//     pub has_daily_gimme_v2: bool,
//     pub has_deal: bool,
//     pub holes: Vec<u32>,
//     pub id: String,
//     pub is_available: bool,
//     pub local_date_time: String,
//     pub location: Option<String>,
//     pub lock_expiration: Option<String>,
//     pub players_display: String,
//     pub players_max: u32,
//     pub players_min: u32,
//     pub primary_prices: Vec<GolfBackPrimaryPrices>,
//     pub rates: Vec<GolfBackRate>,
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GolfBackRate {
//     pub base_price: f64,
//     pub description: String,
//     pub fee_display: f64,
//     pub has_cart_included: bool,
//     pub holes: u32,
//     pub is_daily_gimme_v2: bool,
//     pub is_deal: bool,
//     pub is_gimme: bool,
//     pub is_primary: bool,
//     pub name: String,
//     pub price: f64,
//     pub rate_plan_id: String,
//     pub use_primary_after_selection: bool,
// }
