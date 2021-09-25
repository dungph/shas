use super::DB;
use serde_json::{Map, Value};
use sqlx::{query, Result};

pub async fn _upsert_data(entity: &[u8], data: Map<String, Value>) -> Result<()> {
    if data.is_empty() {
        Ok(())
    } else {
        query!(
            r#"
            -- UPSERT VALUE
            insert into entity(public_key, entity_data)
            values($1, $2)
            on conflict(public_key, entity_data) do update
            set entity_data = entity.entity_data || $2
            "#,
            entity,
            Value::Object(data)
        )
        .execute(&*DB)
        .await?;
        Ok(())
    }
}

pub async fn _get_data(entity: &[u8]) -> Result<Map<String, Value>> {
    Ok(query!(
        r#"
        -- GET ENTITY'S DATA 
        select entity_data from entity 
        where public_key = $1
        "#,
        entity
    )
    .fetch_one(&*DB)
    .await?
    .entity_data
    .as_object()
    .unwrap_or(&Map::new())
    .clone()
    .into_iter()
    .collect())
}
