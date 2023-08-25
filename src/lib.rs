use chrono::{Local, NaiveDateTime, Utc};
use sqlx::{query, query_as, FromRow, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
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

#[derive(FromRow, Debug, Clone, Serialize,Deserialize)]
pub struct Poll {
    pub id : i64,
    pub poll_name: String,
    pub open : bool,
    pub close_date : Option<chrono::NaiveDateTime>,
    #[sqlx(rename = "type")]
    pub r#type : i64,
}
#[derive(FromRow, Debug, Clone,Serialize,Deserialize)]
pub struct House {
    pub id : i64,
    pub house_name: String,
    pub address : String,
    pub image: String
}
#[derive(FromRow, Debug, Clone,Serialize,Deserialize)]
pub struct PollHouse {
    pub poll_id : i64,
    pub house_id : i64,
    pub player_name : String,
    pub ip : String,
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

        // sqlx::migrate!("./migrations").run(&db_con).await.unwrap();

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

        let mut bulk_db = appst.db.begin().await.map_err(TheErrors::DatabaseError)?;

        for mem in &new_data.free_company_members {
            let in_old_state = oldm_st.iter().find(|old| old.id == mem.id);
            if let Some(fm) = in_old_state {
                // member already in db, let's check if they changed rank
                if fm.rank != mem.rank {
                    println!("updating rank for {:?} -> {:?}", mem.name, mem.rank);
                    sqlx::query!("UPDATE fcMembers SET rank = ? WHERE id = ?", mem.rank, mem.id)
                        .execute(&mut *bulk_db)
                        .await
                        .map_err(TheErrors::DatabaseError)?;
                }
                // check if they left and then rejoined?
                if fm.left {
                    println!("updating left status for {:?} -> {:?}", mem.name, fm.left);
                    sqlx::query!("UPDATE fcMembers SET (left,leftDate) = (0,NULL) WHERE id = ?", mem.id)
                        .execute(&mut *bulk_db)
                        .await
                        .map_err(TheErrors::DatabaseError)?;
                }
            }
            else {
                // brand new member
                println!("adding new member -> {:?}", mem.name);
                sqlx::query!("INSERT OR IGNORE INTO fcMembers (id, name, rank,avatar,entryDate, left) VALUES (?,?,?,?,(SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now'))),0)",mem.id, mem.name, mem.rank,mem.avatar)
                    .execute(&mut *bulk_db)
                    .await
                    .map_err(TheErrors::DatabaseError)?;
            }
        }
        // check for members in db, but not in latest data
        let members_gone = oldm_st.iter().filter(|old| !new_data.free_company_members.iter().any(|new| new.id == old.id)).cloned().collect::<Vec<FCMem>>();
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
        let new_members =
            members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FCMem>>();


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
    seconds > (120 * 60)
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
                if let Err(e) = update_db(&mut uw).await {
                    println!("[TASK]error updating db {:?}", e);
                };
            }
        }
    }
}