mod structs;
mod course_service;
mod tee_time_service;

use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    routing::get,
    Router,
    response::Json,
    extract::{
        State,
        Query
    }
};

use course_service::courses::{
    load_courses,
    get_courses,
};

use tee_time_service::get_tee_times;

use structs::{
    AppState,
    GolfCourse,
    Coords,
    TeeTime,
};


#[tokio::main]
async fn main() {
    let courses = load_courses();

    let state = AppState {
        courses: Arc::new(courses),
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/courses", get(courses_handler))
        .route("/tee_times", get(tee_times_handler))
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // run it
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello from Rust!"
}


async fn courses_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<GolfCourse>> {
    let coords = params
        .get("coords")
        .and_then(|value| serde_json::from_str::<Coords>(value).ok());

    // get_courses returns Vec<&GolfCourse>, so we clone each course
    let result: Vec<GolfCourse> = get_courses(&state.courses, coords)
        .into_iter()
        .cloned()
        .collect();

    Json(result)
}


async fn tee_times_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<TeeTime>> {
    let date: String = params
        .get("date")
        .cloned()
        .unwrap_or_else(|| chrono::Local::now()
            .format("%Y-%m-%d")
            .to_string()
        );

    let players = params
        .get("players")
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(4);

    println!("RAW coords param: {:?}", params.get("coords"));

    let coords = params
        .get("coords")
        .and_then(|c| serde_json::from_str::<Coords>(c).ok());

    let filtered_courses= get_courses(&state.courses, coords);

    let tee_times = get_tee_times(
        &filtered_courses, 
        &date, 
        players, 
    ).await;

    Json(tee_times)
}
