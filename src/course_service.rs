use std::fs;

use crate::structs::{
    GolfCourse,
    Coords,
};

pub mod courses {
    pub use super::*;

    pub fn load_courses() -> Vec<GolfCourse> {
        let data = fs::read_to_string("./src/resources/golf_courses.json")
            .expect("Failed to read golf_courses.json");

        serde_json::from_str(&data)
            .expect("Invalid JSON format")
    }

    pub fn default_coords() -> Coords {
        Coords {
            min_lat: 38.757,
            max_lat: 39.427,
            min_lon: -94.908,
            max_lon: -94.235,
        }
    }

    // pub fn get_courses(courses: &[GolfCourse], coords: Option<Coords>) -> Vec<GolfCourse> {
    //     let coords = coords.unwrap_or_else(default_coords);

    //     courses
    //         .iter()
    //         .filter(|course| {
    //             coords.min_lat <= course.lat &&
    //             course.lat <= coords.max_lat &&
    //             coords.min_lon <= course.lon &&
    //             course.lon <= coords.max_lon
    //         })
    //         .cloned()
    //         .collect()
    // }

    pub fn get_courses<'a>(
        courses: &'a [GolfCourse],
        coords: Option<Coords>,
    ) -> Vec<&'a GolfCourse> {
        let coords = coords.unwrap_or_else(default_coords);

        courses
            .iter()
            .filter(|course| {
                coords.min_lat <= course.lat &&
                course.lat <= coords.max_lat &&
                coords.min_lon <= course.lon &&
                course.lon <= coords.max_lon
            })
            // .cloned()
            .collect()
    }
}