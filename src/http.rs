use super::{api::Response, creds::Login};
use ntex::web::{get, Error as WebError, HttpRequest, HttpResponse};
use ntex_files::NamedFile;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub async fn fourofour() -> Result<HttpResponse, WebError> {
    let mut content = String::new();

    let fourofour_path = Path::new("./html").join("notfound.html");

    if fourofour_path.is_file() {
        let mut fourofour_file = File::open(fourofour_path)?;
        fourofour_file.read_to_string(&mut content)?;
        return Ok(HttpResponse::NotFound()
            .content_type("text/html")
            .body(content));
    }

    return Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body("<h1> 404 Not Found <h1>"));
}

#[get("/")]
pub async fn index() -> Result<HttpResponse, WebError> {
    let mut content = String::new();
    let index_path = Path::new("./html").join("index.html");

    let mut file = File::open(index_path)?;
    file.read_to_string(&mut content)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}

#[get("/privacy")]
pub async fn privacy() -> Result<HttpResponse, WebError> {
    let mut content = String::new();
    let privacy_path = Path::new("./html").join("privacy.html");

    let mut file = File::open(privacy_path)?;
    file.read_to_string(&mut content)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}

#[get("/contact")]
pub async fn contact() -> Result<HttpResponse, WebError> {
    let mut content = String::new();
    let contact_path = Path::new("./html").join("contact.html");

    let mut file = File::open(contact_path)?;
    file.read_to_string(&mut content)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}

#[get("/login")]
pub async fn login() -> Result<HttpResponse, WebError> {
    let mut content = String::new();
    let login_path = Path::new("./html").join("login.html");

    let mut file = File::open(login_path)?;
    file.read_to_string(&mut content)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}

#[get("/dashboard")]
pub async fn dashboard(session: ntex_session::Session) -> Result<HttpResponse, WebError> {
    let credentials = Login::get();
    let mut content = String::new();
    let dashboard_path = Path::new("./html").join("dashboard.html");
    let mut file = File::open(dashboard_path)?;
    file.read_to_string(&mut content)?;

    if let Some(hash) = session.get::<String>("hash")? {
        if let Some(user) = session.get::<String>("user")? {
            if credentials.verify(hash.clone(), user) {
                return Ok(HttpResponse::Ok().content_type("text/html").body(content));
            }
        }
    }

    Ok(HttpResponse::Unauthorized()
        .content_type("application/json")
        .json(&Response {
            title: "Unauthorized".to_string(),
            message: "Failed to verify token.".to_string(),
        }))
}

pub async fn files(req: HttpRequest) -> Result<HttpResponse, WebError> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    let file = NamedFile::open(PathBuf::from("./").join(path));
    if file.is_ok() {
        return Ok(file?.into_response(&req));
    }
    fourofour().await
}
