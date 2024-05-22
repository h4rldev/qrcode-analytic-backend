use super::{
    creds::Login,
    data::{read_from_json, write_to_json, AppData, AppState, JsonData, JsonState},
};
use bcrypt::{hash, DEFAULT_COST};

use chrono::{prelude::*, Duration};
use ntex::{
    http::header::HeaderValue,
    web::{
        get, post,
        types::{Json, State},
        Error as WebError, HttpRequest, HttpResponse,
    },
};
use ntex_session::Session;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/*
 * struct SuccessResponse {
 *   message: String,
 *   time: String,
 *   last_time: String,
 *   counter: i32,
 * }
 *
 * Response struct for constructing json response when request succeeds.
 */

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
    time: String,
    last_time: String,
    counter: i32,
}

/*
 * struct BlockedResponse {
 *   message: String,
 *   current_count: i32,
 *   last_time: String,
 * }
 *
 * Response struct for constructing json response when request is blocked.
 */

#[derive(Serialize)]
struct BlockedResponse {
    message: String,
    current_count: i32,
    last_time: String,
}

/*
 * struct Response {
 *   pub title: String,
 *   pub message: String,
 * }
 *
 * Default response used for authentication mostly.
 */

#[derive(Serialize)]
pub struct Response {
    pub title: String,
    pub message: String,
}

/*
 * struct RoutingResponse {
 *   title: String,
 *   message: String,
 *   route: String,
 * }
 *
 * A different iteration of Response that also specifies the route that you should redirect to.
 * (It was easier doing this than using Location headers.)
 */

#[derive(Serialize)]
struct RoutingResponse {
    title: String,
    message: String,
    route: String,
}

/*
 * struct LoginPost {
 *   username: String,
 *   password: String,
 * }
 *
 * The JSON request data struct used in authentication, easier to parse than default FORM data.
 */

#[derive(Deserialize, Serialize)]
struct LoginPost {
    username: String,
    password: String,
}

/*
 * async fn can_user_enter(session: ntex_session::Session) -> Result<bool, WebError> {}
 *
 * Checks user session time & updates it, if 22 hours have gone since last visit,
 * it updates it and returns true.
 * else it returns false.
 */
async fn can_user_enter(session: ntex_session::Session) -> Result<bool, WebError> {
    if session.get::<String>("session_time")?.is_some() {
        let time_since_last_visit: String = session
            .get("session_time")?
            .expect("Can't get cookie despite being some.");
        let time_here =
            DateTime::parse_from_rfc3339(&time_since_last_visit).expect("Can't parse from rfc3339");
        let time_difference = Local::now().signed_duration_since(time_here);
        if time_difference < Duration::try_hours(22).expect("Can't get hours") {
            return Ok(false);
        }
    } else {
        session.set("session_time", Local::now().to_rfc3339())?;
    }
    Ok(true)
}

/*
 * https://url.tld/api
 *
 * Main api point, checks cookies and updates counter and times.
 * also updates state.
 */

#[get("/api")]
pub async fn main_endpoint(
    session: Session,
    data: State<Arc<Mutex<AppData>>>,
    req: HttpRequest,
) -> Result<HttpResponse, WebError> {
    if req.headers().get("Request-Source").is_none()
        && req.headers().get("Request-Source") != Some(&HeaderValue::from_static("qrcode-analytic"))
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let current_date = Local::now().date_naive().to_string();
    let current_time = Local::now().time().format("%H:%M:%S").to_string();
    let current_dotw = Local::now().weekday().to_string();
    let data = &mut data.lock().await.state;
    let current_data = data.last().expect("Can't get latest entry");

    let blocked = BlockedResponse {
        message: "You've already checked in, come back tomorrow!".to_string(),
        current_count: current_data.counter,
        last_time: format!(
            "{} - {}, {}",
            current_data.time.clone(),
            current_data.dotw.clone(),
            current_data.date.clone()
        ),
    };

    if !can_user_enter(session).await? {
        return Ok(HttpResponse::AlreadyReported().json(&blocked));
    }

    let success = SuccessResponse {
        message: "Success, you can now close this page, or check out other data below.".to_string(),
        time: format!("{} - {}, {}", current_time, current_dotw, current_date),
        last_time: format!(
            "{} - {}, {}",
            current_data.time.clone(),
            current_data.dotw.clone(),
            current_data.date.clone()
        ),
        counter: current_data.counter + 1,
    };

    let index_of_yesterday = if data.len() != 1 { data.len() - 2 } else { 0 };
    let count_since_yesterday = current_data.counter - data[index_of_yesterday].counter;

    let new_data = AppState {
        last_date: current_data.date.clone(),
        date: current_date,
        counter: current_data.counter + 1,
        dotw: current_dotw,
        count_since_yesterday,
        last_time: current_data.time.clone(),
        time: current_time.clone(),
    };

    if new_data.date == current_data.date {
        let last_app_state = data.last_mut().expect("Can't get latest entry");
        last_app_state.time.clone_from(&new_data.time);
        last_app_state.last_time = new_data.last_time;
        last_app_state.counter = new_data.counter;
        last_app_state.count_since_yesterday = count_since_yesterday;
    } else {
        data.push(new_data);
    }

    let json_data = JsonData { state: Vec::new() };
    let mut json_state = json_data.state;

    for entry in data {
        json_state.push(JsonState {
            date: entry.date.clone(),
            last_count: entry.counter,
            count_since_yesterday: entry.count_since_yesterday,
            dotw: entry.dotw.clone(),
            last_time: entry.time.clone(),
        })
    }

    let json_data = JsonData { state: json_state };

    let path = Path::new("./state");

    write_to_json(path, json_data).await?;
    Ok(HttpResponse::Ok().json(&success))
}

/*
 * https://url.tld/api/get_data
 *
 * Returns state info for dashboard.
 * Checks if authenticated and such.
 */

#[get("/api/get_data")]
async fn get_state(
    req: HttpRequest,
    session: ntex_session::Session,
) -> Result<HttpResponse, WebError> {
    if req.headers().get("Request-Source").is_none()
        && req.headers().get("Request-Source") != Some(&HeaderValue::from_static("qrcode-analytic"))
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let credentials = Login::get();
    if let Some(hash) = session.get::<String>("hash")? {
        if let Some(user) = session.get::<String>("user")? {
            if !credentials.verify(hash, user) {
                return Ok(HttpResponse::Unauthorized()
                    .content_type("application/json")
                    .json(&Response {
                        title: "Unauthorized".to_string(),
                        message: "You're not allowed to retrieve this data.".to_string(),
                    }));
            }
        }
    }

    let current_dir = std::env::current_dir()?;
    let state_path = current_dir.join("state");

    let last_data = if read_from_json(&state_path).await.is_ok() {
        read_from_json(&state_path).await?.state
    } else {
        JsonData::default().state
    };

    let app_data = AppData { state: Vec::new() };
    let mut app_state = app_data.state;

    for entry in last_data {
        app_state.push(AppState {
            date: entry.date.clone(),
            last_date: entry.date,
            counter: entry.last_count,
            count_since_yesterday: entry.count_since_yesterday,
            dotw: entry.dotw,
            time: entry.last_time.clone(),
            last_time: entry.last_time,
        })
    }

    let new_data = AppData { state: app_state };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(&new_data))
}

/*
 * https://url.tld/login - POST
 * The function that parses and verifies the data send when pressing submit on the login form.
 * Uses cookies and doesn't need to be used often since it saves your login and autoredirects you.
 */

#[post("/login")]
pub async fn authenticate(
    req: HttpRequest,
    json: Json<LoginPost>,
    session: ntex_session::Session,
) -> Result<HttpResponse, WebError> {
    if req.headers().get("Request-Source").is_none()
        && req.headers().get("Request-Source") != Some(&HeaderValue::from_static("qrcode-analytic"))
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let credentials = Login::get();
    let hashed = hash(&json.password, DEFAULT_COST).expect("Can't hash password");
    if !credentials.verify(hashed.clone(), json.username.clone()) {
        return Ok(HttpResponse::Unauthorized()
            .content_type("application/json")
            .json(&Response {
                title: "Unauthorized".to_string(),
                message: "Invalid username or password.".to_string(),
            }));
    }

    session.set("hash", hashed.clone())?;
    session.set("user", json.username.clone())?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(&RoutingResponse {
            title: "Logged in!".to_string(),
            message: "Redirecting to dashboard...".to_string(),
            route: "/dashboard".to_string(),
        }))
}

/*
 * https://url.tld/api/can_i_login
 *
 * Helping function that the login page requests to for auto login.
 */

#[get("/api/can_i_login")]
pub async fn can_login(
    req: HttpRequest,
    session: ntex_session::Session,
) -> Result<HttpResponse, WebError> {
    if req.headers().get("Request-Source").is_none()
        && req.headers().get("Request-Source") != Some(&HeaderValue::from_static("qrcode-analytic"))
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let credentials = Login::get();
    if let Some(hash) = session.get::<String>("hash")? {
        if let Some(user) = session.get::<String>("user")? {
            if credentials.verify(hash, user) {
                return Ok(HttpResponse::Ok().content_type("application/json").json(
                    &RoutingResponse {
                        title: "You're already logged in!".to_string(),
                        message: "Redirecting to dashboard...".to_string(),
                        route: "/dashboard".to_string(),
                    },
                ));
            }
        }
    }

    session.set("hash", "".to_string())?;
    session.set("user", "".to_string())?;

    return Ok(HttpResponse::Unauthorized()
        .content_type("application/json")
        .json(&Response {
            title: "Couldn't auto login".to_string(),
            message: "Has your credentials reset?, Resetting your cookies.".to_string(),
        }));
}
