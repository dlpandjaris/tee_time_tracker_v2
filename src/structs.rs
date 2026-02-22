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

// #[derive(Debug, Deserialize)]
// pub struct GolfBackPrimaryPrices {
//     #[serde(rename = "basePrice")]
//     pub base_price: f64,
//     pub holes: i64,
//     pub price: f64,
// }

#[derive(Debug, Deserialize)]
pub struct GolfBackTeeTime {
    // pub courseId: String,
    // pub courseName: String,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    // pub has9Holes: bool,
    // pub hasDailyGimmeV2: bool,
    // pub hasDeal: bool,
    pub holes: Vec<u32>,
    pub id: String,
    // pub isAvailable: bool,
    // pub localDateTime: String,
    // pub location: Option<String>,
    // pub lockExpiration: Option<String>,
    // pub playersDisplay: String,
    #[serde(rename = "playersMax")]
    pub players_max: u32,
    // #[serde(rename = "playersMin")]
    // pub players_min: u32,
    // pub primaryPrices: Vec<GolfBackPrimaryPrices>,
    pub rates: Vec<GolfBackRate>,
}

#[derive(Debug, Deserialize)]
pub struct GolfBackRate {
    // #[serde(rename = "basePrice")]
    // pub base_price: f64, // 34.0,
    // pub description: String, // "All you can play, up to 18 holes",
    // pub feeDisplay: f64, // 0.0,
    // pub hasCartIncluded: bool, // true,
    // pub holes: u32, // 18,
    // pub isDailyGimmeV2: bool, // false,
    // pub isDeal: bool, // false,
    // pub isGimme: bool, // false,
    // pub isPrimary: bool, // true,
    // pub name: String, // "Twilight",
    pub price: f64, // 34.0,
    #[serde(rename = "ratePlanId")]
    pub rate_plan_id: String, // "0bc2bf83-2bf5-4ba1-be8b-a06691bf761a",
    // pub usePrimaryAfterSelection: bool, // false
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


#[derive(Debug, Deserialize)]
pub struct TeeItUpResponse {
    pub teetimes: Vec<TeeItUpTeeTime>,
}

#[derive(Debug, Deserialize)]
pub struct TeeItUpTeeTime {
    pub teetime: String,
    #[serde(rename = "maxPlayers")]
    pub max_players: u32,
    pub rates: Vec<TeeItUpRate>,
}

#[derive(Debug, Deserialize)]
pub struct TeeItUpRate {
    pub holes: u32,
    #[serde(rename = "greenFeeCart")]
    pub green_fee_cart: Option<u64>,
    pub promotion: Option<TeeItUpPromotion>,
}

#[derive(Debug, Deserialize)]
pub struct TeeItUpPromotion {
    #[serde(rename = "greenFeeCart")]
    pub green_fee_cart: u64,
}