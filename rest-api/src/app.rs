use crate::config::{BirdeyeConfig, DatabaseConfig};
use crate::price::{self, PriceSdk};
use crate::thirdparty::alternative_api::AlternativeClient;
use crate::thirdparty::BirdEyeClient;
use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use envconfig::Envconfig;
use http_body_util::BodyExt;
use sqlx::{Pool, Postgres};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
pub struct AppState {
    pub version: i32,
    pub bird_eye_client: BirdEyeClient,
    pub alternative_client: AlternativeClient,
    pub pool: Pool<Postgres>,
}
const ALTERNATIVE_BASE_URL: &str = "https://api.alternative.me/fng/";
pub const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";

impl AppState {
    pub async fn new() -> Self {
        init_tracing();
        let BirdeyeConfig { api_key, base_url } = BirdeyeConfig::init_from_env().unwrap();
        Self {
            version: 0,
            bird_eye_client: BirdEyeClient::new(&base_url, &api_key),
            alternative_client: AlternativeClient::new(ALTERNATIVE_BASE_URL.into(), 31),
            pool: init_pg_pool().await,
        }
    }

    pub async fn start_worker(&self) {
        let sched = JobScheduler::new().await.unwrap();
        let pool = self.pool.clone();
        let client = self.bird_eye_client.clone();

        sched
            .add(
                Job::new_async("0 * * * * *", move |_uuid, mut _l| {
                    Box::pin(async move {
                        info!("running");
                    })

                    //let client = client.clone();
                    //let pool = pool.clone();
                    //Box::pin(async move {
                    //    info!("Refresh sol price");
                    //    let Ok(metric) = client.get_price(SOL_ADDRESS).await else {
                    //        return;
                    //    };
                    //
                    //    if let Err(err) =
                    //        price::store_metric_in_db(&pool, &metric, SOL_ADDRESS).await
                    //    {
                    //        error!("store metric {err}");
                    //    }
                    //})
                })
                .unwrap(),
            )
            .await
            .unwrap();
        if let Err(err) = sched.start().await {
            error!("start cron job erro {err}");
        }
    }
}

async fn init_pg_pool() -> Pool<Postgres> {
    let DatabaseConfig { database_url, .. } = DatabaseConfig::init_from_env().unwrap();
    // 2) Create a connection pool
    let pool = Pool::<Postgres>::connect(&database_url)
        .await
        .expect("should create pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("should run successfully");
    pool
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

pub async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
