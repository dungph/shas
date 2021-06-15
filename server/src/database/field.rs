use super::DB;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{query, Result};
use std::collections::BTreeMap;
use utils::{Field, Fields};

pub async fn upsert_field(entity: &str, field: &str, data: &Field) -> Result<()> {
    if data.is_empty() {
        Ok(())
    } else {
        query!(
            r#"
            -- UPSERT VALUE, META
            insert into field(entity, name, data)
            values($1, $2, $3)
            on conflict(entity, name) do update
            set data = field.data || $3
            "#,
            entity,
            field,
            Value::Object(data.clone())
        )
        .execute(&*DB)
        .await?;
        Ok(())
    }
}

pub async fn get_fields(entity: &str) -> Result<Fields> {
    let all = query!(
        r#"
        -- GET ENTITY'S FIELDS
        select name, data from field 
        where entity = $1
        "#,
        entity
    )
    .fetch_all(&*DB)
    .await?;
    Ok(all
        .iter()
        .map(|db_field| {
            (
                db_field.name.clone(),
                db_field.data.as_object().unwrap().clone(),
            )
        })
        .collect())
}

pub async fn _get_field(entity: &str, field: &str) -> Result<Option<Field>> {
    Ok(query!(
        r#"
        -- GET FIELD
        select data 
        from field 
        where entity = $1 
        and name = $2
        "#,
        entity,
        field
    )
    .fetch_optional(&*DB)
    .await?
    .map(|obj| obj.data.as_object().unwrap().clone()))
}

pub async fn get_log_checked(
    entity: &str,
    field: &str,
    for_entity: &str,
) -> Result<BTreeMap<DateTime<Utc>, Value>> {
    Ok(query!(
        r#"
        -- GET LOG
        select field_log.data , field_log.timest
        from field_log
        inner join field_subscription
            on field_subscription.object = field_log.entity
            and field_subscription.field = field_log.field
        where field_log.entity = $1
        and field_log.field= $2
        and field_subscription.subject = $3
        and field_subscription.read = true
        "#,
        entity,
        field,
        for_entity
    )
    .fetch_all(&*DB)
    .await?
    .iter()
    .filter_map(|o| o.data.as_ref().map(|v| (o.timest.clone(), v.clone())))
    .collect())
}

pub async fn get_reading_fields(for_entity: &str) -> Result<BTreeMap<String, Fields>> {
    let mut ret = BTreeMap::new();
    query!(
        r#"
        -- GET READING FIELD
        select field.entity, field.name, field.data
        from field
        inner join field_subscription
            on field_subscription.object = field.entity
            and field_subscription.field = field.name
        where field_subscription.read = true
        and field_subscription.subject = $1
        "#,
        for_entity
    )
    .fetch_all(&*DB)
    .await?
    .iter()
    .for_each(|obj| {
        *ret.entry(obj.entity.clone())
            .or_insert(Fields::new())
            .entry(obj.name.clone())
            .or_insert(Field::new()) = obj.data.as_object().unwrap().clone()
    });
    Ok(ret)
}
