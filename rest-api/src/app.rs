use crate::config::{BirdeyeConfig, DatabaseConfig};
use crate::fearandgreed::{batch_insert_fear_and_greed, FearAndGreed, FearAndGreedSdk};
use crate::price::{self, PriceSdk, TimeFilters};
use crate::thirdparty::alternative_api::AlternativeClient;
use crate::thirdparty::defi::DefiClient;
use crate::thirdparty::{BirdEyeClient, MoniClient};
use crate::token::TokenSdk;
use crate::{time_util, token, volume};
use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use envconfig::Envconfig;
use helius::client::HeliusAsyncSolanaClient;
use helius::types::Cluster;
use helius::Helius;
use http_body_util::BodyExt;
use reqwest::Client;
use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub version: i32,
    pub bird_eye_client: BirdEyeClient,
    pub alternative_client: AlternativeClient,
    pub moni_client: Arc<MoniClient>,
    pub pool: Pool<Postgres>,
    pub client: reqwest::Client,
    // pub helius: Arc<Helius>,
}
const ALTERNATIVE_BASE_URL: &str = "https://api.alternative.me/fng/";
pub const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const SOLANA: &str = "solana";

impl AppState {
    pub async fn new() -> Self {
        init_tracing();
        let BirdeyeConfig {
            birdeye_api_key,
            base_url,
            moni_api_key,
        } = BirdeyeConfig::init_from_env().unwrap();
        // let helius_api_key: &str = "your_api_key";
        // let cluster: Cluster= Cluster::MainnetBeta;
        let client = Client::new();
        Self {
            version: 0,
            bird_eye_client: BirdEyeClient::new(&base_url, &birdeye_api_key),
            alternative_client: AlternativeClient::new(ALTERNATIVE_BASE_URL.into(), 31),
            pool: init_pg_pool().await,
            moni_client: Arc::new(MoniClient::new(moni_api_key, client.clone())),
            client, // helius: Arc::new(Helius::new(api_key, cluster).unwrap()),
        }
    }

    pub async fn start_worker(app: Arc<Self>) {
        let sched = JobScheduler::new().await.unwrap();
        let pool = app.pool.clone();
        let alternative_client = app.alternative_client.clone();
        let app1 = app.clone();
        Self::run(
            &pool,
            &DefiClient {
                client: Client::new(),
            },
            &alternative_client,
        )
        .await;
        sched
            .add(
                Job::new_async("0 0 * * * *", move |_uuid, mut _l| {
                    let app = app1.clone();
                    let defi_client = DefiClient {
                        client: Client::new(),
                    };
                    let alternative_client = alternative_client.clone();
                    Box::pin(async move {
                        AppState::run(&app.pool, &defi_client, &alternative_client).await;
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();

        let app2 = app.clone();
        Self::token_price_histories(&app2).await;
        sched
            .add(
                Job::new_async("0 0 * * * *", move |_uuid, mut _l| {
                    let app = app2.clone();
                    Box::pin(async move {
                        Self::token_price_histories(&app).await;
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();
        let app3 = app.clone();
        Self::sol_price(&app3).await;
        sched
            .add(
                Job::new_async("0 0 * * * *", move |_uuid, mut _l| {
                    let app = app3.clone();
                    Box::pin(async move {
                        Self::sol_price(&app).await;
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();

        let mindshare_state = app.clone();
        Self::mind_share(&app).await;
        sched
            .add(
                Job::new_async("0 0 * * * *", move |_uuid, mut _l| {
                    let app = mindshare_state.clone();
                    Box::pin(async move {
                        Self::mind_share(&app).await;
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();

        if let Err(err) = sched.start().await {
            error!("start cron job error {err}");
        }
    }

    async fn token_price_histories(app: &Arc<AppState>) {
        match app
            .bird_eye_client
            .get_price_by_time_filter(SOL_ADDRESS, TimeFilters::OneDay)
            .await
        {
            Err(e) => error!("upsert Refresh metric error {e}"),
            Ok(sol_price) => {
                match price::insert_token_prices(&app.pool, sol_price, SOL_ADDRESS).await {
                    Ok(_) => info!("refresh price history successfully"),
                    Err(err) => error!("refresh price history failed {err}"),
                };
            }
        };
    }

    async fn run(pool: &PgPool, defi_client: &DefiClient, alternative_client: &AlternativeClient) {
        let Ok(resp) = defi_client.get_blockchain_volum(SOLANA).await else {
            return;
        };

        info!("fetching volume {resp:?}");
        if let Err(e) = volume::upsert_metrics(pool, resp, SOLANA).await {
            error!("upsert Refresh metric error {e}");
            return;
        }

        let Ok(resp) = alternative_client.get_fear_and_greed(31).await else {
            return;
        };

        let result = resp.iter().map(FearAndGreed::from).collect();
        info!("fetching volume {result:?}");
        if let Err(e) = batch_insert_fear_and_greed(pool, &result).await {
            error!("upsert Refresh metric error {e}");
            return;
        }
    }

    async fn sol_price(p0: &Arc<AppState>) {
        // ✅ 2. If not found, call 3rd-party API
        let resp = p0.bird_eye_client.get_price(SOL_ADDRESS).await;
        let Ok(metric) = resp else {
            error!("fetch price error {}", resp.unwrap_err());
            return;
        };

        // ✅ 3. Store the metric in the database
        if let Err(err) = price::store_metric_in_db(&p0.pool, &metric, SOL_ADDRESS).await {
            error!("store price failed {err}");
        };
        info!("fetch price stored successfully");
    }

    async fn mind_share(state: &Arc<AppState>) {
        match state.bird_eye_client.get_trending(0, 20).await {
            Ok(trending_token) => {
                info!("get trending tokens {trending_token:?}");
                if let Err(e) = token::upsert_token_meta(&state.pool, &trending_token).await {
                    error!("upsert trending token error {e}");
                }
                let record_at = time_util::get_start_of_day(Utc::now()).timestamp();
                if let Err(e) =
                    token::upsert_daily_volume(&state.pool, &trending_token, record_at).await
                {
                    error!("upsert trending token error {e}");
                }
            }
            Err(err) => {
                error!("get trending tokens {err}");
            }
        };
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
