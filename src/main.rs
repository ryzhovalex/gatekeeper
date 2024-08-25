use corund_lib::get_router;
use log::info;

#[tokio::main]
async fn main() {
    colog::init();

    // let args: Vec<String> = env::args().collect();

    info!("start server http://0.0.0.0:3000");
    let listener =
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, get_router()).await.unwrap();
}
