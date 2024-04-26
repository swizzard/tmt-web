use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tmt_web::{make_app, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tmt_web=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = make_app(AppState::from_env());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
