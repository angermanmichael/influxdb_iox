//! Implementation of command line option for running router

use super::main;
use clap_blocks::object_store::make_object_store;
use clap_blocks::{
    catalog_dsn::CatalogDsnConfig, run_config::RunConfig, write_buffer::WriteBufferConfig,
};
use iox_time::{SystemProvider, TimeProvider};
use ioxd_common::{
    server_type::{CommonServerState, CommonServerStateError},
    Service,
};
use ioxd_router::create_router_server_type;
use object_store::DynObjectStore;
use object_store_metrics::ObjectStoreMetrics;
use observability_deps::tracing::*;
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

const QUERY_POOL_NAME: &str = "iox-shared";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Run: {0}")]
    Run(#[from] main::Error),

    #[error("Invalid config: {0}")]
    InvalidConfig(#[from] CommonServerStateError),

    #[error("Cannot parse object store config: {0}")]
    ObjectStoreParsing(#[from] clap_blocks::object_store::ParseError),

    #[error("Creating router: {0}")]
    Router(#[from] ioxd_router::Error),

    #[error("Catalog DSN error: {0}")]
    CatalogDsn(#[from] clap_blocks::catalog_dsn::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, clap::Parser)]
#[clap(
    name = "run",
    about = "Runs in router mode",
    long_about = "Run the IOx router server.\n\nThe configuration options below can be \
    set either with the command line flags or with the specified environment \
    variable. If there is a file named '.env' in the current working directory, \
    it is sourced before loading the configuration.

Configuration is loaded from the following sources (highest precedence first):
        - command line arguments
        - user set environment variables
        - .env file contents
        - pre-configured default values"
)]
pub struct Config {
    #[clap(flatten)]
    pub(crate) run_config: RunConfig,

    #[clap(flatten)]
    pub(crate) catalog_dsn: CatalogDsnConfig,

    /// Query pool name to dispatch writes to.
    #[clap(
        long = "--query-pool",
        env = "INFLUXDB_IOX_QUERY_POOL_NAME",
        default_value = "iox-shared",
        action
    )]
    pub(crate) query_pool_name: String,

    /// The maximum number of simultaneous requests the HTTP server is
    /// configured to accept.
    ///
    /// This number of requests, multiplied by the maximum request body size the
    /// HTTP server is configured with gives the rough amount of memory a HTTP
    /// server will use to buffer request bodies in memory.
    ///
    /// A default maximum of 200 requests, multiplied by the default 10MiB
    /// maximum for HTTP request bodies == ~2GiB.
    #[clap(
        long = "--max-http-requests",
        env = "INFLUXDB_IOX_MAX_HTTP_REQUESTS",
        default_value = "200",
        action
    )]
    pub(crate) http_request_limit: usize,

    /// The location InfluxDB IOx will use to store files locally. If not specified, will run in
    /// ephemeral mode.
    #[clap(long = "--data-dir-router", env = "INFLUXDB_IOX_DB_DIR", action)]
    pub database_directory_router: Option<PathBuf>,
}

pub async fn command(config: Config) -> Result<()> {
    let common_state = CommonServerState::from_config(config.run_config.clone())?;
    let time_provider = Arc::new(SystemProvider::new()) as Arc<dyn TimeProvider>;
    let metrics = Arc::new(metric::Registry::default());

    let write_buffer_config =
        WriteBufferConfig::new(QUERY_POOL_NAME, config.database_directory_router);

    let catalog = config
        .catalog_dsn
        .get_catalog("router", Arc::clone(&metrics))
        .await?;

    let object_store = make_object_store(config.run_config.object_store_config())
        .map_err(Error::ObjectStoreParsing)?;
    // Decorate the object store with a metric recorder.
    let object_store: Arc<DynObjectStore> = Arc::new(ObjectStoreMetrics::new(
        object_store,
        time_provider,
        &*metrics,
    ));

    info!("starting router");
    let server_type = create_router_server_type(
        &common_state,
        Arc::clone(&metrics),
        catalog,
        object_store,
        &write_buffer_config,
        QUERY_POOL_NAME,
        1_000,
    )
    .await?;

    info!("starting router");
    let services = vec![Service::create(server_type, common_state.run_config())];
    Ok(main::main(common_state, services, metrics).await?)
}
