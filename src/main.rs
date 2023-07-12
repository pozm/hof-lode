use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;


async fn the_the() -> &'static str {
    "The the"
}

#[tokio::main]
async fn main() {


    let router = Router::new().route("/", get(the_the));


    axum::Server::bind(&"0.0.0.0:3133".parse().unwrap()).serve(router.into_make_service()).await.unwrap();


}