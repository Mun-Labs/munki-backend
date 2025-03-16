pub mod route;
pub mod model;
pub mod search;

pub use route::*;

pub struct TrendingTokenResponse {}

pub trait TokenSdk {
    fn trending_token(&self) -> TrendingTokenResponse;
}
