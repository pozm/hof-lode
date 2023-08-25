use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, HeaderValue};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};

use hof_lode::{
    should_update, update_db, AppState, FCMem, Poll, TheErrors, run_tasks,
};
use reqwest::header;
use serde_json::json;
use tower_http::set_header::SetResponseHeaderLayer;
use std::sync::Arc;

use tokio::select;
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
async fn the_the(
    State(data): State<ApSt>,
) -> Result<HtmlTemplate<impl Template>, HtmlTemplate<impl Template>> {
    let templ: Result<IndexTemplate, TheErrors> = async {
        let last_update = async {
            let daa = data.read().await;
            daa.last_update
        }
        .await;

        let (mut data, new, gone) = if should_update(last_update) {
            println!("cache miss");
            let mut st = data.write().await;
            update_db(&mut st).await?
        } else {
            println!("cache hit");
            (data.read().await.cache_data.clone(), Vec::new(), Vec::new())
        };

        data.sort_by_key(|a| (a.left, a.rank.clone(), a.entryDate));
        Result::<_, TheErrors>::Ok(IndexTemplate {
            members: data,
            new_members: new,
            gone_members: gone,
        })
    }
    .await;
    match templ {
        Ok(t) => Ok(HtmlTemplate(t)),
        Err(e) => {
            println!("error -> {:?}", e);
            Err(HtmlTemplate(ErrorTemplate {
                error: e.to_string(),
            }))
        }
    }
}

async fn get_polls(State(st): State<ApSt>) -> Json<Vec<Poll>> {
    let db = st.read().await.db.clone();

    let active_polls = sqlx::query_as!(Poll, "SELECT * FROM poll where poll.open = 1")
        .fetch_all(&db)
        .await
        .unwrap();

    Json(active_polls)
}
async fn get_poll_housing_options(
    State(st): State<ApSt>,
    Path(poll_id): Path<u32>,
) -> impl IntoResponse {
    let db = st.read().await.db.clone();

    let sel_poll = sqlx::query!("SELECT house.house_name as name, house.address, house.image, pho.id as option_id FROM poll left join poll_house_option as pho on pho.poll_id = poll.id left join house on pho.house_id = house.id where poll.id = ? ",poll_id).fetch_all(&db).await.unwrap();

    Json(
        sel_poll
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "address": a.address,
                    "image": a.image,
                    "option_id": a.option_id
                })
            })
            .collect::<Vec<_>>(),
    )
}
#[derive(serde::Deserialize)]
struct SubmitHousingVote {
    option_id: u32,
    player_name: String,
}
// #[axum::debug_handler]
async fn submit_housing_vote(
    State(st): State<ApSt>,
    Path(poll_id): Path<u32>,
    hdrs: HeaderMap,
    Json(vote_data): Json<SubmitHousingVote>,
) -> impl IntoResponse {
    let db = st.read().await.db.clone();

    let temp_rd = st.read().await;
    if !temp_rd
        .cache_data
        .iter()
        .any(|mem| mem.name.eq_ignore_ascii_case(&vote_data.player_name))
    {
        return Json(json!({
            "error": "player not found"
        }));
    }
    drop(temp_rd);

    #[cfg(not(debug_assertions))]
    let Some(ip) = hdrs.get("x-real-ip") else {
        return Json(json!({
            "error": "no ip" // should in theory be impossible cause my nginx should set this
        }));
    };
    #[cfg(not(debug_assertions))]
    let Ok(ip) = ip.to_str() else {
        return Json(json!({
            "error": "no ip" // should in theory be impossible cause my nginx should set this
        }));
    };
    #[cfg(debug_assertions)]
    let ip = "::1";

    //  check if poll is open;

    let Ok(Some(close_date)) = sqlx::query!("select open from poll where poll.id = ? ",poll_id).fetch_optional(&db).await else {
        return Json(json!({
            "error": "poll not found"
        }));
    };
    if !close_date.open {
        return Json(json!({
            "error": "poll closed"
        }));
    }

    let Ok(_) = sqlx::query!("INSERT OR IGNORE INTO poll_house (poll_id, option_id, player_name, ip) VALUES (?, ?, ?, ?)", poll_id, vote_data.option_id, vote_data.player_name,ip).execute(&db).await else {
        return Json(json!({
            "error": "db error (probably ip collision)"
        }));
    };

    Json(json!({
        "success": true
    }))
}

#[tokio::main]
async fn main() {
    let data = Arc::new(tokio::sync::RwLock::new(AppState::new().await));

    let router = Router::new()
        .route("/mem", get(the_the))
        .route("/api/polls", get(get_polls))
        .route(
            "/api/polls/housing/:poll_id",
            get(get_poll_housing_options).post(submit_housing_vote),
        )
        .nest_service("/assets", ServeDir::new("./assets/"))
        .nest_service("/p", ServeDir::new("./public/").append_index_html_on_directories(true)).layer(
            tower::ServiceBuilder::new().layer(SetResponseHeaderLayer::if_not_present(
                header::HeaderName::from_static("x-server-name"),
                HeaderValue::from_static("axum"),
            ))
        )
        .with_state(data.clone());

    let tasks_fut = run_tasks(data.clone());
    let server_fut =
        axum::Server::bind(&"0.0.0.0:3133".parse().unwrap()).serve(router.into_make_service());
    select! {
        _ = tasks_fut => {
            println!("tasks stopped");
        },
        _ = server_fut => {
            println!("server stopped");
            data.write().await.stop();
        },
    }
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
