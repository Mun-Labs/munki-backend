use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "RPC_URL")]
    pub rpc_url: String,
}

#[derive(Envconfig)]
pub struct BirdeyeConfig {
    #[envconfig(from = "BIRDEYE_API_KEY")]
    pub birdeye_api_key: String,

    #[envconfig(from = "MONI_API_KEY")]
    pub moni_api_key: String,

    #[envconfig(from = "BIRDEYE_API_URL")]
    pub base_url: String,
}

#[derive(Envconfig)]
pub struct DatabaseConfig {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    #[envconfig(from = "RUN_MIGRATION", default = "true")]
    pub run_miration: bool,
}
