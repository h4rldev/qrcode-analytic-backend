use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};
use std::{
    fs::{create_dir, File},
    path::Path,
};

/*
 * pub struct JsonData {
 *   pub state: Vec<JsonState>,
 * }
 *
 * Struct used for writing the state to JSON, the JSON being an Vector (Dynamic Array).
 */

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonData {
    pub state: Vec<JsonState>,
}

/*
 * pub struct JsonState {
 *   pub date: String,
 *   pub dotw: String, < dotw stands for Day of the week.
 *   pub last_count: i32,
 *   pub count_since_yesterday: i32,
 *   pub last_time: String,
 * }
 *
 * All data that gets read from JSON and written to JSON.
 */

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonState {
    pub date: String,
    pub dotw: String,
    pub last_count: i32,
    pub count_since_yesterday: i32,
    pub last_time: String,
}

/*
 * pub struct AppData {
 *   pub state: Vec<AppState>,
 * }
 *
 * Struct used for managing Data read and written to under the entire program.
 * Holds the data in a Vector (Dynamic Array)
 */

#[derive(Clone, Serialize)]
pub struct AppData {
    pub state: Vec<AppState>,
}

/*
 * pub struct AppState {
 *   pub last_date: String,
 *   pub date: String,
 *   pub dotw: String,
 *   pub counter: i32,
 *   pub count_since_yesterday: i32,
 *   pub time: String,
 *   pub last_time: String,
 * }
 *
 * Struct used to hold all data collected and used when the service is running.
 */

#[derive(Clone, Serialize)]
pub struct AppState {
    pub last_date: String,
    pub date: String,
    pub dotw: String,
    pub counter: i32,
    pub count_since_yesterday: i32,
    pub time: String,
    pub last_time: String,
}

/*
 * impl Default for JsonState {}
 *
 * Initializes JsonState for convenience.
 */

impl Default for JsonState {
    fn default() -> Self {
        JsonState {
            date: Local::now().date_naive().to_string(),
            last_count: 0_i32,
            dotw: Local::now().weekday().to_string(),
            count_since_yesterday: 0_i32,
            last_time: Local::now().time().to_string(),
        }
    }
}

/*
 * impl Default for JsonData {}
 *
 * Initializes JsonData for convenience.
 */

impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            state: vec![JsonState::default()],
        }
    }
}

/*
 * async fn read_from_json(path: &Path) -> Result<JsonData, std::io::Error> {}
 *
 * Parses and returns json data as JsonData.
 */

pub async fn read_from_json(path: &Path) -> Result<JsonData, std::io::Error> {
    if !path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    if !file_path.is_file() {
        return Err(std::io::ErrorKind::NotFound.into());
    }
    let file = File::open(file_path)?;
    let json_data: JsonData = from_reader(file)?;
    Ok(json_data)
}

/*
 * async fn write_to_json(path: &Path, json_data: JsonData) -> Result<(), std::io::Error> {}
 *
 * Writes JsonData to a json file.
 */

pub async fn write_to_json(path: &Path, json_data: JsonData) -> Result<(), std::io::Error> {
    if !path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    let file = File::create(file_path)?;
    to_writer(&file, &json_data)?;
    Ok(())
}
