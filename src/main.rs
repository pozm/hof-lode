use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::Local;
use hof_lode::json::FreeCompanyMember;
use hof_lode::{fetch_new_data, should_update, AppState, FCMem, update_db, TheErrors, run_tasks};
use sqlx::{query, query_as, Acquire, Executor};
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
async fn the_the(State(data): State<ApSt>) -> Result<HtmlTemplate<impl Template>,HtmlTemplate<impl Template>> {
    let templ : Result<IndexTemplate,TheErrors> = async {
        let last_update = async {
            let daa = data.read().await;
            daa.last_update.clone()
        }.await;

        let (mut data, new, gone) = if should_update(last_update) {
            println!("cache miss");
            let mut st=  data.write().await;
            update_db(&mut *st).await?
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
        .with_state(data.clone());

    let tasks_fut = run_tasks(data.clone());
    let server_fut = axum::Server::bind(&"0.0.0.0:3133".parse().unwrap())
        .serve(router.into_make_service());
    select! {
        _ = tasks_fut => {},
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
