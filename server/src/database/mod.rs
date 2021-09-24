mod account;
mod entity;

use once_cell::sync::Lazy;

static DB: Lazy<sqlx::PgPool> = Lazy::new(|| {
    let url = &*crate::vars::DB_URL;
    sqlx::PgPool::connect_lazy(url).unwrap()
});

pub async fn migrate() -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(&*DB).await?;
    Ok(())
}
