use chrono::{Local, NaiveDateTime, Utc};
use sqlx::{query, query_as, FromRow, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::select;
use tokio::sync::RwLock;
use tokio::time::interval;
use crate::json::FreeCompanyMember;

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
    pub stopped: bool,
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
            stopped: false,
        }
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
}
#[derive(thiserror::Error, Debug)]
pub enum TheErrors {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("fetch error: {0}")]
    FetchError(#[from] reqwest::Error),
}


pub async fn update_db(appst : &mut AppState) -> Result<(Vec<FCMem>, Vec<FCMem>,Vec<FCMem>),TheErrors> {
        let new_data = fetch_new_data().await.map_err(TheErrors::FetchError)?;
        println!("updating data -> {:?}", new_data.free_company.name);

        let oldm_st = appst.cache_data.clone();

        let members_gone = oldm_st.iter().filter(|old| !old.left && !new_data.free_company_members.iter().any(|new| new.id == old.id)).cloned().collect::<Vec<FCMem>>();
        let new_members = new_data.free_company_members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FreeCompanyMember>>();

        let mut bulk_db = appst.db.begin().await.map_err(TheErrors::DatabaseError)?;

        for mem in &new_members {
            println!("adding new member -> {:?}", mem.name);
            sqlx::query!("INSERT OR IGNORE INTO fcMembers (id, name, rank,avatar,entryDate, left) VALUES (?,?,?,?,(SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now'))),0)",mem.id, mem.name, mem.rank,mem.avatar)
                .execute(&mut *bulk_db)
                .await
                .map_err(TheErrors::DatabaseError)?;
        }
        for mem in &members_gone {
            println!("removing member -> {:?}", mem.name);
            sqlx::query!("UPDATE fcMembers SET (left,leftDate) = (1,(SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now')))) WHERE id = ?", mem.id)
                .execute(&mut *bulk_db)
                .await
                .map_err(TheErrors::DatabaseError)?;
        }

        query!("UPDATE updateTime SET last_update = (SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now'))) WHERE id = 1")
            .execute(&mut *bulk_db)
            .await
            .map_err(TheErrors::DatabaseError)?;

        bulk_db.commit().await.map_err(TheErrors::DatabaseError)?;

        let members = query_as!(FCMem, "SELECT * FROM fcMembers")
            .fetch_all(&appst.db)
            .await
            .map_err(TheErrors::DatabaseError)?;
        let new_members = if new_members.len() < 10 {
            members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FCMem>>()
        } else {
            Vec::new()
        };

    appst.cache_data = members.clone();
    appst.last_update = Local::now().naive_local();
    Ok((members, new_members, members_gone))
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
pub async fn run_tasks(state : Arc<RwLock<AppState>>) {
    let mut invt = interval(Duration::from_secs(60 * 60));
    loop {
        let uw = state.read().await;
        if uw.stopped {
            break;
        }
        drop(uw);
        select! {
            _ = invt.tick() => {
                println!("[TASK]updating db");
                let mut uw = state.write().await;
                if let Err(e) = update_db(&mut *uw).await {
                    println!("[TASK]error updating db {:?}", e);
                };
            }
        }
    }
}