use std::{env, net::Ipv4Addr};

use lazy_static::lazy_static;
use serde::Deserialize;

// Env variables
lazy_static! {
    // datatabse config
    pub static ref DB_USERNAME: String = env::var("DB_USERNAME").unwrap();
    pub static ref DB_PASSWORD: String = env::var("DB_PASSWORD").unwrap();
    pub static ref DB_HOST: String = env::var("DB_HOST").unwrap();
    pub static ref DB_PORT: u16 = env::var("DB_PORT").unwrap().parse().unwrap();
    pub static ref DB_NAME: String = env::var("DB_NAME").unwrap();

    // server config
    pub static ref SERVER_HOST: Ipv4Addr = env::var("SERVER_HOST").unwrap().parse().unwrap();
    pub static ref SERVER_PORT: u16 = env::var("SERVER_PORT").unwrap().parse().unwrap();

    // other params
    pub static ref PAGE_SIZE: i64 = env::var("PAGE_SIZE").unwrap().parse().unwrap();
    pub static ref ABIS_FOLDER: String = env::var("ABIS_FOLDER").unwrap();
    pub static ref PASSWORD_SALT: String = env::var("PASSWORD_SALT").unwrap();

    pub static ref X_EX_APIKEY: String = env::var("X_EX_APIKEY").unwrap();
    pub static ref X_EX_SECRETKEY: String = env::var("X_EX_SECRETKEY").unwrap();

    pub static ref P2WAY_APIKEY: String = env::var("P2WAY_APIKEY").unwrap();
    pub static ref P2WAY_SECRETKEY: String = env::var("P2WAY_SECRETKEY").unwrap();
    pub static ref P2WAY_SECRETKEY_HASH: String = env::var("P2WAY_SECRETKEY_HASH").unwrap();

    pub static ref HCAPTCHA_SECRET: String = env::var("HCAPTCHA_SECRET").unwrap();

    pub static ref ENGINES: u16 = env::var("ENGINES").unwrap().parse().unwrap();

    pub static ref GOOGLE_CLIENT_ID: String = env::var("GOOGLE_CLIENT_ID").unwrap();
    pub static ref GOOGLE_SECRET_KEY: String = env::var("GOOGLE_SECRET_KEY").unwrap();

    pub static ref BILLINE_MERCHANT: String = env::var("BILLINE_MERCHANT").unwrap();
    pub static ref BILLINE_SECRET: String = env::var("BILLINE_SECRET").unwrap();
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
