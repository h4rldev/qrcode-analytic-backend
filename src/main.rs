use api::{authenticate, can_login, get_state, main_endpoint};
use creds::Login;
use data::{read_from_json, AppData, AppState, JsonData};
use http::{contact, dashboard, files, index, login, privacy};

use ntex::web::{get, middleware, App, HttpServer};
use ntex_session::CookieSession;

use std::sync::Arc;
use tokio::sync::Mutex;

mod api;
mod creds;
mod data;
mod http;

/*
 * Main function, the base of the entire website as a whole
 * Manages all site structure and initializes state on startup.
 */

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install().expect("Can't install hooks.");

    let _ = Login::get(); // Make sure to generate a login.

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

    let state = Arc::new(Mutex::new(AppData { state: app_state }));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::default()
                .header("Content-Security-Policy", 
                    "default-src 'self'; style-src 'self'; img-src 'self' https://http.cat/ data: blob:; font-src 'self' https://fonts.bunny.net/")
            )
            .service(index)
            .service(privacy)
            .service(login)
            .service(dashboard)
            .service(contact)
            .service(main_endpoint)
            .service(get_state)
            .service(can_login)
            .route("/{filename}*", get().to(files))
            .service(authenticate)
            .state(state.clone())
            .wrap(
                CookieSession::private(&[0; 128])
                    .name("qrcode")
                    .secure(false),
            )
            .wrap(ntex::web::middleware::Logger::default())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
