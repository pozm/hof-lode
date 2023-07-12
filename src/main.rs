use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::Local;
use hof_lode::json::FreeCompanyMember;
use hof_lode::{fetch_new_data, should_update, AppState, FCMem};
use sqlx::{query, query_as, Acquire, Executor};
use std::sync::Arc;
use tower_http::services::ServeDir;

type ApSt = Arc<tokio::sync::RwLock<AppState>>;

#[derive(Template)]
#[template(path = "index.askama", escape = "none")]
struct IndexTemplate {
    new_members: Vec<FCMem>,
    gone_members: Vec<FCMem>,
    members: Vec<FCMem>,
}
#[derive(Template)]
#[template(path = "error.askama", escape = "none")]
struct ErrorTemplate {
    error: String,
}
#[derive(thiserror::Error, Debug)]
enum TheErrors {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("fetch error: {0}")]
    FetchError(#[from] reqwest::Error),
}

async fn the_the(State(data): State<ApSt>) -> Result<HtmlTemplate<impl Template>,HtmlTemplate<impl Template>> {
    let templ : Result<IndexTemplate,TheErrors> = async {
        let (last_update, oldm_st) = async {
            let daa = data.read().await;
            (daa.last_update.clone(), daa.cache_data.clone())
        }.await;

        let (mut data, new, gone) = if should_update(last_update) {
            println!("cache miss");

            let mut wd = data.write().await;
            let new_data = fetch_new_data().await.map_err(TheErrors::FetchError)?;
            println!("updating data -> {:?}", new_data.free_company.name);

            let members_gone = oldm_st.iter().filter(|old| !new_data.free_company_members.iter().any(|new| new.id == old.id)).cloned().collect::<Vec<FCMem>>();
            let new_members = new_data.free_company_members.iter().filter(|new| !oldm_st.iter().any(|old| new.id == old.id)).cloned().collect::<Vec<FreeCompanyMember>>();

            let mut bulk_db = wd.db.begin().await.map_err(TheErrors::DatabaseError)?;

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
                .fetch_all(&wd.db)
                .await
                .map_err(TheErrors::DatabaseError)?;
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
        Result::<_,TheErrors>::Ok(IndexTemplate {
            members: data,
            new_members: new,
            gone_members: gone
        })
    }.await;
    return match templ {
        Ok(t) => {
            Ok(HtmlTemplate(t))
        },
        Err(e) => {
            println!("error -> {:?}", e);
            Err(HtmlTemplate(ErrorTemplate {
                error: e.to_string(),
            }))
        }
    }
}

#[tokio::main]
async fn main() {
    let data = Arc::new(tokio::sync::RwLock::new(AppState::new().await));

    let router = Router::new()
        .route("/", get(the_the))
        .nest_service("/assets", ServeDir::new("./assets/"))
        .with_state(data);

    axum::Server::bind(&"0.0.0.0:3133".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
