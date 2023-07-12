
use std::sync::Arc;
use askama::Template;
use axum::extract::{ State};
use axum::{ Router};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use chrono::{Local};
use sqlx::{Acquire, Executor, query, query_as,};
use tower_http::services::ServeDir;
use hof_lode::{AppState, FCMem, fetch_new_data, should_update};
use hof_lode::json::FreeCompanyMember;

type ApSt = Arc<tokio::sync::RwLock<AppState>>;


#[derive(Template)]
#[template(path = "index.askama", escape = "none")]
struct IndexTemplate {
    new_members: Vec<FCMem>,
    gone_members: Vec<FCMem>,
    members: Vec<FCMem>
}
async fn the_the(State(data) : State<ApSt>) -> HtmlTemplate<IndexTemplate> {

    let (last_update,oldm_st) = async {
        let daa = data.read().await;
        (daa.last_update.clone(),daa.cache_data.clone())
    }.await;

    let (mut data,new,gone) = if should_update(last_update) {

        println!("cache miss");

        let mut wd = data.write().await;
        let new_data = fetch_new_data().await.unwrap();
        println!("updating data -> {:?}", new_data.free_company.name);

        let members_gone = oldm_st.iter().filter(|old| !new_data.free_company_members.iter().any(|new| new.id == old.id)).cloned().collect::<Vec<FCMem>>();
        let new_members = new_data.free_company_members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FreeCompanyMember>>();

        let mut bulk_db = wd.db.begin().await.unwrap();

        for mem in &new_members {
            println!("adding new member -> {:?}", mem.name);
            sqlx::query!("INSERT OR IGNORE INTO fcMembers (id, name, rank,avatar,entryDate, left) VALUES (?,?,?,?,(SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now'))),0)",mem.id, mem.name, mem.rank,mem.avatar)
                .execute(&mut *bulk_db)
                .await
                .unwrap();
        }
        for mem in &members_gone {
            println!("removing member -> {:?}", mem.name);
            sqlx::query!("UPDATE fcMembers SET (left,leftDate) = (1,(SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now')))) WHERE id = ?", mem.id)
                .execute(&mut *bulk_db)
                .await
                .unwrap();
        }

        query!("UPDATE updateTime SET last_update = (SELECT strftime('%Y-%m-%d %H:%M:%S', datetime('now'))) WHERE id = 1")
            .execute(&mut *bulk_db)
            .await
            .unwrap();

        bulk_db.commit().await.unwrap();

        let members = query_as!(FCMem, "SELECT * FROM fcMembers")
            .fetch_all(&wd.db)
            .await
            .unwrap();
        let new_members = if new_members.len() < 10 {
            members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FCMem>>()
        } else {
            Vec::new()
        };

        wd.cache_data = members.clone();
        wd.last_update = Local::now().naive_local();
        (members, new_members, members_gone)
    } else {
        println!("cache hit");
        (data.read().await.cache_data.clone(), Vec::new(), Vec::new())
    };
    data.sort_by_key(|a| a.left);
    let templt = IndexTemplate {
        members: data,
        new_members : new,
        gone_members : gone
    };
    HtmlTemplate( templt )
}

#[tokio::main]
async fn main() {

    let data =  Arc::new(tokio::sync::RwLock::new(AppState::new().await));

    let router = Router::new()
        .route("/", get(the_the))
        .nest_service("/assets", ServeDir::new("./assets/"))
        .with_state(data);


    axum::Server::bind(&"0.0.0.0:3133".parse().unwrap()).serve(router.into_make_service()).await.unwrap();


}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => {
                Html(html).into_response()
            },
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}