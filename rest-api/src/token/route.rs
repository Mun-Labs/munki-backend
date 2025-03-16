use crate::app::AppState;
use crate::response::HttpResponse;
use crate::token::query_top_token_volume_history;
use axum::extract::{Query, State};
use axum::{http::StatusCode, Json};
use bigdecimal::ToPrimitive;
use serde::Serialize;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthyResponse {
    pub message: String,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenMindshare {
    token_address: String,
    change_percentage: f64,
    logo_url: String,
    pub name: String,
    pub symbol: String,
}
pub async fn mindshare(
    State(app): State<AppState>,
) -> Result<Json<HttpResponse<Vec<TokenMindshare>>>, (StatusCode, String)> {
    let vol = query_top_token_volume_history(&app.pool, 20)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total_volume: f64 = vol
        .iter()
        .map(|v| v.volume24h.to_f64().unwrap_or_default())
        .sum();
    let percent = vol
        .iter()
        .map(|v| TokenMindshare {
            token_address: v.token_address.clone(),
            change_percentage: (v.volume24h.to_f64().unwrap_or_default() / total_volume) * 100.0,
            logo_url: v.logo_uri.clone().unwrap_or_default(),
            name: v.name.clone(),
            symbol: v.symbol.clone(),
        })
        .collect::<Vec<_>>();
    Ok(Json(HttpResponse {
        code: 200,
        response: percent,
    }))
}

// rust
use anyhow::Result;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: i64,
    pub offset: i64,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Token {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
}

pub async fn search_token(
    State(app): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Token>>, (StatusCode, String)> {
    let tokens = search_tokens(&app.pool, &query.q, query.limit, query.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(tokens))
}
pub async fn search_tokens(
    pool: &Pool<Postgres>,
    search: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Token>> {
    // Use full-text search by concatenating name and symbol into a tsvector and comparing against a tsquery.

    let tokens = sqlx::query_as::<_, Token>(
        r#"SELECT token_address, name, symbol, image_url as logo_uri FROM tokens WHERE token_address % $1 OR name % $1 OR symbol % $1 limit $2 offset $3"#,
    )
        .bind(search)
        .bind(limit).bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(tokens)
}
