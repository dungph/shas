use once_cell::sync::Lazy;
use std::env::var;

pub static WEB_PORT: Lazy<String> = Lazy::new(|| {
    if let Some(s) = var("PORT").ok() {
        s
    } else {
        String::from("8080")
    }
});

pub static DB_URL: Lazy<String> = Lazy::new(|| {
    let url = var("DATABASE_URL");
    url.expect("set DATABASE_URL to your postgres uri")
});

pub static SECRET: Lazy<String> = Lazy::new(|| {
    let pwd = var("PRIVATE");
    pwd.unwrap_or(String::from_utf8_lossy(&rand::random::<[u8; 32]>()).into())
});

pub static ADMIN: Lazy<String> = Lazy::new(|| {
    let pwd = var("ADMIN_PWD");
    pwd.unwrap_or(String::from("admin"))
});
