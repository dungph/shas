use super::DB;
use argon2::{hash_encoded, verify_encoded, Config};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, Result};

pub async fn _create_account(username: &str, password: &str, admin: bool) -> Result<PgQueryResult> {
    let conf = Config::default();
    let salt = rand::random::<[u8; 32]>();
    let pwd = hash_encoded(password.as_bytes(), &salt, &conf).unwrap();

    query!(
        r#"
        -- CREATE ENTITY
        insert into user_account (username, password, admin)
        values ($1, $2, $3);
        "#,
        username,
        pwd,
        admin,
    )
    .execute(&*DB)
    .await
}

pub async fn _get_account(entity: &str, password: &str) -> Result<Option<bool>> {
    query!(
        r"
        -- GET ENTITY 
        select password, admin from user_account  
        where username=$1
        ",
        entity
    )
    .fetch_optional(&*DB)
    .await
    .map(|maybe_obj| {
        maybe_obj
            .map(|obj| (obj.password, obj.admin))
            .map(|(pwd, admin)| {
                if verify_encoded(&pwd, password.as_bytes()).unwrap_or(false) {
                    Some(admin)
                } else {
                    None
                }
            })
            .flatten()
    })
}
