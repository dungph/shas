use super::DB;
use sqlx::{query, Result};

pub async fn get_readers(entity: &str, field: &str) -> Result<Vec<String>> {
    let mut all = query!(
        r#"
        -- GET FIELD'S READERS
        select subject from field_subscription 
        where (object = $1 and field = $2 and read = true)
        "#,
        entity,
        field
    )
    .fetch_all(&*DB)
    .await?;
    Ok(all.iter_mut().map(|o| o.subject.clone()).collect())
}

pub async fn _get_readings(entity: &str) -> Result<Vec<(String, String)>> {
    let mut all = query!(
        r#"
        -- GET READING FIELDS FOR ENTITY
        select * from field_subscription
        where subject = $1
        "#,
        entity
    )
    .fetch_all(&*DB)
    .await?;
    Ok(all
        .iter_mut()
        .map(|o| (o.object.clone(), o.field.clone()))
        .collect())
}

pub async fn get_requesters(entity: &str, field: &str) -> Result<Vec<String>> {
    let mut all = query!(
        r#"
        -- GET FIELD'S WRITER (REQUEST SENDER)
        select subject from field_subscription 
        where (object = $1 and field = $2 and request = true)"#,
        entity,
        field
    )
    .fetch_all(&*DB)
    .await?;
    Ok(all.iter_mut().map(|o| o.subject.clone()).collect())
}
