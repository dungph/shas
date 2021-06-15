use super::DB;
use argon2::{hash_encoded, Config};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as, Result};
use utils::Entity;

pub async fn create_entity(entity: &Entity) -> Result<PgQueryResult> {
    let conf = Config::default();
    let salt = rand::random::<[u8; 32]>();
    let pwd = hash_encoded(entity.password.as_bytes(), &salt, &conf).unwrap();

    query!(
        r#"
        -- CREATE ENTITY
        insert into entity (name, password, passphrase, admin)
        values ($1, $2, $3, $4);
        "#,
        entity.name,
        pwd,
        entity.passphrase,
        entity.admin,
    )
    .execute(&*DB)
    .await
}

pub async fn create_entity_by(entity: &Entity, by: &str) -> Result<PgQueryResult> {
    create_entity(entity).await?;
    query!(
        r#"
        -- SUBSCRIPTION FOR NEW ENTITY
        insert into entity_subscription(
            subject,
            object,
            manage,
            read,
            request
            )
        values ($1, $2, $3, $3, $3)
        "#,
        by,
        entity.name,
        true
    )
    .execute(&*DB)
    .await
}

pub async fn get_entity(entity: &str) -> Result<Option<Entity>> {
    query_as!(
        Entity,
        r"
        -- GET ENTITY 
        select * from entity 
        where name=$1
        ",
        entity
    )
    .fetch_optional(&*DB)
    .await
}
