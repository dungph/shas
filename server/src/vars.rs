use once_cell::sync::Lazy;
use sha2::Digest;
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

pub static PKEY: Lazy<[u8; 32]> = Lazy::new(|| {
    var("PRIVATE")
        .map(|s| {
            let mut sha256 = sha2::Sha256::default();
            sha256.update(s);
            sha256.finalize().into()
        })
        .unwrap_or(rand::random::<[u8; 32]>())
});
