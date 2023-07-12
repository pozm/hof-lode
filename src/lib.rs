use chrono::{Local, NaiveDateTime, Utc};
use sqlx::{query, query_as, FromRow, SqlitePool};
use std::collections::HashMap;
use std::time::SystemTime;

pub mod json;

#[derive(FromRow, Debug, Clone)]
#[sqlx(rename_all = "camelCase")]
pub struct FCMem {
    pub id: i64,
    pub name: String,
    pub rank: String,
    pub avatar: String,
    #[sqlx(rename = "entryDate")]
    pub entryDate: Option<chrono::NaiveDateTime>,
    pub left: bool,
    #[sqlx(rename = "leftDate")]
    pub leftDate: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub last_update: chrono::NaiveDateTime,
    pub cache_data: Vec<FCMem>,
    pub db: sqlx::SqlitePool,
}

impl AppState {
    pub async fn new() -> Self {
        let db_con = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite:./hof_lode.db?mode=rwc")
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&db_con).await.unwrap();

        let members = query_as!(FCMem, "SELECT * FROM fcMembers")
            .fetch_all(&db_con)
            .await
            .unwrap();

        let last_update = query!("select last_update from updateTime where id = 1")
            .fetch_one(&db_con)
            .await
            .unwrap()
            .last_update
            .unwrap();

        Self {
            cache_data: members,
            last_update,
            db: db_con,
        }
    }
}

pub async fn fetch_new_data() -> reqwest::Result<json::Root> {
    let the_data = reqwest::get("https://xivapi.com/freecompany/9235053248388316377?data=FCM")
        .await?
        .json::<json::Root>()
        .await?;

    Ok(the_data)
}
pub fn should_update(last: NaiveDateTime) -> bool {
    let now = Local::now().naive_local();
    let duration = now.signed_duration_since(last);
    let seconds = duration.num_seconds();
    return seconds > (120 * 60);
}
