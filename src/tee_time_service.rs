use regex::Regex;
use scraper::{Html, Selector};
use chrono::{NaiveDateTime, Utc, TimeZone, DateTime};
use chrono_tz::America::Chicago;
use reqwest::{
    Client,
    Response
};

use futures::future::join_all;

use crate::structs::{
    TeeTime,
    GolfCourse,
    CourseId,
    GolfBackResponse,
    GolfBackTeeTime,
    GolfBackRate,
    ForeUpTeeTime,
    TeeItUpResponse,
};

pub mod book_a_tee_time {
    pub use super::*;

    // BookATeeTime
    pub async fn fetch(
        client: &Client,
        course: &GolfCourse,
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        let course_id: String = match &course.id {
            CourseId::Number(n) => n.to_string(),
            CourseId::String(s) => s.clone(),
            CourseId::Verbose(v) => v.id.to_string(),
        };

        let url: String = format!(
            "https://bookateetime.teequest.com/search/{}/{date}?selectedPlayers={players}&selectedHoles=18",
            course_id
        );

        let response: Response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[bookateetime] HTTP error {}: {}", course.name, e);
                return vec![];
            }
        };

        let body = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[bookateetime] Body error {}: {}", course.name, e);
                return vec![];
            }
        };

        let document: Html = Html::parse_document(&body);
        let tee_time_selector: Selector = Selector::parse("div.tee-time").unwrap();
        let holes_selector: Selector = Selector::parse("span").unwrap();
        let link_selector: Selector = Selector::parse("a.btn").unwrap();
        let holes_re: Regex = Regex::new(r"\d+").unwrap();

        document
            .select(&tee_time_selector)
            .filter_map(|div| {
                let holes = div
                    .select(&holes_selector)
                    .find_map(|s| {
                        let text = s.text().collect::<String>();
                        holes_re.find(&text)?.as_str().parse::<u32>().ok()
                    });

                let tee_time_str = div.value().attr("data-date-time")?;
                let price: f64 = div.value().attr("data-price")?.parse().ok()?;
                let players_avail: u32 = div.value().attr("data-available")?.parse().ok()?;

                let naive = NaiveDateTime::parse_from_str(tee_time_str, "%Y%m%d%H%M").ok()?;
                
                // Interpret as Chicago time (CST/CDT aware)
                let central_dt = Chicago.from_local_datetime(&naive).single()?;
                let tee_time = central_dt.with_timezone(&Utc);

                let href = div
                    .select(&link_selector)
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .unwrap_or("");

                Some(TeeTime {
                    course: course.name.clone(),
                    tee_time,
                    price,
                    players: players_avail,
                    holes,
                    lat: course.lat,
                    lon: course.lon,
                    book_url: format!("https://bookateetime.teequest.com{}", href),
                })
            })
            .collect()
    }


    pub async fn search(
        courses: &[&GolfCourse],
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        let client = Client::new();

        let tasks: Vec<_> = courses
            .iter()
            .filter(|c| c.source == "bookateetime")
            .map(|course| self::fetch(&client, course, date, players))
            .collect();

        join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}


pub mod golfback {
    use super::*;

    pub async fn fetch(
        client: &Client,
        course: &GolfCourse,
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        let course_id = match &course.id {
            CourseId::Number(n) => n.to_string(),
            CourseId::String(s) => s.clone(),
            CourseId::Verbose(v) => v.id.to_string(),
        };

        let url = format!(
            "https://api.golfback.com/api/v1/courses/{}/date/{}/teetimes",
            course_id, date
        );

        let body = serde_json::json!({
            "date": date,
            "course_id": course_id,
            "players": players
        });

        let response = match client
            .post(&url)
            .header("User-Agent", "Mozilla/5.0")
            .header("Referer", "https://golfback.com/")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[GolfBack] {} HTTP error: {}", course.name, e);
                return vec![];
            }
        };

        let raw_json: serde_json::Value = match response.json().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("[GolfBack] {} JSON decode error: {}", course.name, e);
                return vec![];
            }
        };

        // Now try parsing into struct
        let parsed: GolfBackResponse = match serde_json::from_value(raw_json.clone()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[GolfBack] {} STRUCT PARSE ERROR: {}", course.name, e);
                return vec![];
            }
        };

        parsed
            .data
            .into_iter()
            .filter_map(|tt: GolfBackTeeTime| {
                let first_rate: &GolfBackRate = tt.rates.first()?;
                let parsed_dt = chrono::DateTime::parse_from_rfc3339(&tt.date_time).ok()?;
                let tee_time = parsed_dt.with_timezone(&Utc);

                Some(TeeTime {
                    course: course.name.clone(),
                    tee_time,
                    price: first_rate.price,
                    players: tt.players_max,
                    holes: tt.holes.into_iter().max(),
                    lat: course.lat,
                    lon: course.lon,
                    book_url: format!(
                        "https://golfback.com/#/course/{}/date/{}/teetime/{}?rateId={}&holes=18&players={}",
                        course_id,
                        date,
                        tt.id,
                        first_rate.rate_plan_id,
                        players
                    ),
                })
            })
        .collect()
    }

    pub async fn search(
        courses: &[&GolfCourse],
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        let client = Client::new();

        let tasks = courses
            .iter()
            .filter(|c| c.source == "golfback")
            .map(|course| self::fetch(&client, course, date, players));

        join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}


pub mod foreup {
    use super::*;

    pub async fn fetch(
        client: &Client,
        course: &GolfCourse,
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        // Convert date to MM-DD-YYYY for ForeUp
        let flip_date = match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(d) => d.format("%m-%d-%Y").to_string(),
            Err(e) => {
                eprintln!("[ForeUp] {} date parse error: {}", course.name, e);
                return vec![];
            }
        };

        let url = format!(
            "https://foreupsoftware.com/index.php/api/booking/times?time=all&date={}&holes=all&players={}&booking_class=14824&schedule_id={}&api_key=no_limits",
            flip_date, players, match &course.id {
                CourseId::Number(n) => n.to_string(),
                CourseId::String(s) => s.clone(),
                CourseId::Verbose(v) => v.id.to_string(),
            }
        );

        let response = match client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .header("Referer", format!("https://foreupsoftware.com/index.php/booking/{}/7340", match &course.id {
                CourseId::Number(n) => n.to_string(),
                CourseId::String(s) => s.clone(),
                CourseId::Verbose(v) => v.id.to_string(),
            }))
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[ForeUp] {} HTTP error: {}", course.name, e);
                return vec![];
            }
        };

        let raw_json: serde_json::Value = match response.json().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("[ForeUp] {} JSON decode error: {}", course.name, e);
                return vec![];
            }
        };

        // let body_text = response.text().await.unwrap_or_default();
        // println!("[ForeUp] {} RAW RESPONSE:\n{}", course.name, body_text);

        // Parse into a vector of ForeUpTeeTime
        let parsed: Vec<ForeUpTeeTime> = match serde_json::from_value(raw_json) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ForeUp] {} STRUCT PARSE ERROR: {}", course.name, e);
                return vec![];
            }
        };

        parsed
            .into_iter()
            .filter_map(|tt| {
                let parsed_dt = chrono::NaiveDateTime::parse_from_str(&tt.time, "%Y-%m-%d %H:%M").ok()?;
                let tee_time = Utc.from_utc_datetime(&parsed_dt);

                Some(TeeTime {
                    course: course.name.clone(),
                    tee_time,
                    price: tt.green_fee + tt.cart_fee,
                    players: tt.available_spots,
                    holes: Some(tt.holes),
                    lat: course.lat,
                    lon: course.lon,
                    book_url: format!(
                        "https://foreupsoftware.com/index.php/booking/22857/{}#/teetimes",
                        match &course.id {
                            CourseId::Number(n) => n.to_string(),
                            CourseId::String(s) => s.clone(),
                            CourseId::Verbose(v) => v.id.to_string(),
                        }
                    ),
                })
            })
            .collect()
    }

    pub async fn search(
        courses: &[&GolfCourse],
        date: &str,
        players: u32,
    ) -> Vec<TeeTime> {
        let client = Client::new();

        let tasks = courses
            .iter()
            .filter(|c| c.source == "foreup")
            .map(|course| self::fetch(&client, course, date, players));

        join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}

pub mod teeitup {
    use super::*;

    pub async fn fetch(
        client: &Client,
        course: &GolfCourse,
        date: &str,
        // players: u32,
    ) -> Vec<TeeTime> {
        let verbose = match &course.id {
            CourseId::Verbose(v) => v,
            _ => return vec![],
        };

        let response = match client
            .get("https://phx-api-be-east-1b.kenna.io/v2/tee-times")
            .query(&[
                ("date", date),
                ("facilityIds", &verbose.id.to_string()),
            ])
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", &verbose.url)
            .header("Referer", &verbose.url)
            .header("User-Agent", "Mozilla/5.0")
            .header("X-Be-Alias", &verbose.alias)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[TeeItUp] {} HTTP error: {}", course.name, e);
                return vec![];
            }
        };

        let raw_json: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[TeeItUp] {} JSON parse error: {}", course.name, e);
                return vec![];
            }
        };

        // API returns an array where [0]['teetimes']
        let first = match raw_json.get(0) {
            Some(v) => v,
            None => return vec![],
        };

        let parsed: TeeItUpResponse = match serde_json::from_value(first.clone()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[TeeItUp] {} STRUCT parse error: {}", course.name, e);
                return vec![];
            }
        };

        parsed
            .teetimes
            .into_iter()
            .filter_map(|tt| {
                let rate = tt.rates.first()?;

                // Promotion overrides base rate
                let price_cents = if let Some(promo) = &rate.promotion {
                    promo.green_fee_cart
                } else {
                    rate.green_fee_cart?
                };

                let price = price_cents as f64 / 100.0;

                let tee_time = DateTime::parse_from_rfc3339(&tt.teetime)
                    .or_else(|_| DateTime::parse_from_str(&tt.teetime, "%Y-%m-%d %H:%M:%S"))
                    .ok()?
                    .with_timezone(&Utc);

                Some(TeeTime {
                    course: course.name.clone(),
                    tee_time,
                    price,
                    players: tt.max_players,
                    holes: Some(rate.holes),
                    lat: course.lat,
                    lon: course.lon,
                    book_url: format!(
                        "{}/?course={}&date={}&max=9999",
                        verbose.url, verbose.id, date
                    ),
                })
            })
            .collect()
    }

    pub async fn search(
        courses: &[&GolfCourse],
        date: &str,
        // players: u32,
    ) -> Vec<TeeTime> {
        let client = Client::new();

        let tasks = courses
            .iter()
            .filter(|c| c.source == "teeitup")
            .map(|course| fetch(&client, course, date)); //, players));

        join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}


pub async fn get_tee_times(
    courses: &[&GolfCourse],
    date: &str,
    players: u32,
) -> Vec<TeeTime> {
    let mut results = Vec::new();

    results.extend(book_a_tee_time::search(courses, date, players).await);
    results.extend(golfback::search(courses, date, players).await);
    results.extend(foreup::search(courses, date, players).await);
    results.extend(teeitup::search(courses, date).await); //, players).await);
    // later:
    // results.extend(cps::search(...).await);

    results
}

