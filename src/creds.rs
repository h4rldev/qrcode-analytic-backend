use bcrypt::{hash, verify, DEFAULT_COST};
use password_generator::{generate, PasswordType};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};
use std::{
    env::current_dir,
    fs::{File, OpenOptions},
};

/*
 * pub struct Login {
 *   pub username: String,
 *   pub password: String,
 * }
 *
 * The struct that holds generated Username and Password.
 */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Login {
    pub username: String,
    pub password: String,
}

/*
 * impl Login {
 *  pub fn get() -> Self {}
 *  fn new() -> Self {}
 *  pub fn verify(self, password_hash: String, username: String) -> bool {}
 *  pub fn hash(self) -> Self {}
 * }
 *
 * Assorted implementations accessed through `Login::function(args)`
 * Can also be accessed like this aswell:
 * ```rust
 * let login: Login {
 *   username: "Test".to_string(),
 *   password: "password".to_string(),
 * }
 *
 * login.function(args)
 * ```
 */

impl Login {
    /*
     * pub Login::get() -> Self {}
     *
     * If admin_login.json exists it reads the json, parses it,
     * hashes the password, and later returns it.
     *
     * If admin_login.json doesn't exist, it runs Login::new() -> Self {}
     */

    pub fn get() -> Self {
        let current_dir = current_dir().expect("Can't get current directory");
        let path = current_dir.join("admin_login.json");

        if !path.is_file() {
            return Self::new();
        }

        let file = File::open(path).expect("Can't open file.");
        let mut json_data: Login = from_reader(file).expect("Can't read json file.");
        json_data = json_data.hash();
        json_data
    }

    /*
     * Login::new() -> Self {}
     *
     * Generates admin_login.json for you, default username being "Administrator".
     * Password is automatically generated, being 32 characters long.
     */

    fn new() -> Self {
        let current_dir = current_dir().expect("Can't get current directory");
        let path = current_dir.join("admin_login.json");

        let file = if path.is_file() {
            OpenOptions::new()
                .write(true)
                .read(true)
                .open(path)
                .expect("Can't open file.")
        } else {
            File::create_new(path).expect("Can't create file.")
        };

        let mut login = Login {
            username: "Administrator".to_string(),
            password: generate(32, PasswordType::Ascii).expect("Can't generate."),
        };

        if to_writer(&file, &login).is_err() {
            panic!("Failed to generate login.")
        }

        login = login.hash();
        login
    }

    /*
     * pub Login::verify(self, password_hash: String, username: String) -> bool {}
     *
     * Verifies provided username and password_hash with it's own values,
     * and returns a bool whether they match or not.
     */

    pub fn verify(self, password_hash: String, username: String) -> bool {
        if username != self.username
            && !verify(self.password, &password_hash).expect("Can't verify password.")
        {
            return false;
        }
        true
    }

    /*
     * pub Login::hash(self) -> Self {}
     *
     * Hashes the password field of Login.
     */

    pub fn hash(self) -> Self {
        Login {
            username: self.username,
            password: hash(self.password, DEFAULT_COST).expect("Failed to hash password"),
        }
    }
}
