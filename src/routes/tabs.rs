use crate::{
    db::tabs,
    models::{NewTab, Session, Tab},
    types::{AppError, AppState},
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn tabs_router() -> Router<AppState> {
    Router::new()
        .route("/tabs", post(create))
        .route("/tabs/:tab_id", get(get_tab))
}

async fn create(
    State(st): State<AppState>,
    session: Session,
    Json(payload): Json<NewTab>,
) -> Result<Json<Tab>, AppError> {
    if payload.user_id != session.user_id {
        Err(AppError::WrongCredentials)
    } else {
        let conn = st.conn().await?;
        let tab = tabs::new_tab(conn, payload).await?;
        Ok(Json(tab))
    }
}

async fn get_tab(
    State(st): State<AppState>,
    session: Session,
    Path(tab_id): Path<String>,
) -> Result<Json<Tab>, AppError> {
    let conn = st.conn().await?;
    let tab = tabs::get_tab(conn, session.user_id.clone(), tab_id).await?;
    Ok(Json(tab))
}
