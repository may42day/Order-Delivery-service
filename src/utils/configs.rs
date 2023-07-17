use super::{grpc::orders_grpc::orders_server::OrdersServer, permission_policy::Policy};
use crate::{
    middleware::tracing_middleware::init_subscriber,
    resources::postgresql::{establish_connection_pool, DbPool},
    routes::api::config::api_v1_graphql_config,
    services::orders_service::OrdersService,
};
use axum::{routing::IntoMakeService, Server};
use dotenvy::dotenv;
use hyper::server::conn::AddrIncoming;
use structopt::StructOpt;
use tonic::transport::server::Router;
use tracing::info;

#[derive(Debug, StructOpt, Clone)]
pub struct Opt {
    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[structopt(long, env = "BIND_ADDRESS", default_value = "127.0.0.1:8000")]
    pub bind_address: String,

    #[structopt(
        long,
        env = "GRPC_USER_ADDRESS",
        default_value = "http://0.0.0.0:50051"
    )]
    pub grpc_users_address: String,

    #[structopt(long, env = "GRPC_ORDERS_ADDRESS", default_value = "0.0.0.0:50052")]
    pub grpc_orders_address: String,

    #[structopt(
        long,
        env = "GRPC_ANALYTICS_ADDRESS",
        default_value = "http://0.0.0.0:50053"
    )]
    pub grpc_analytics_address: String,

    // During this time user can estimate delivery after it was finished
    // in seconds
    #[structopt(long, env = "DELIVERY_ESTIMATION_TIME", default_value = "600")]
    pub delivery_estimation_time: i32,
}

#[derive(Clone)]
pub struct Config {
    // pub app_state: AppState,
    pub db_pool: DbPool,
    pub permission_policy: Policy,
    pub bind_address: String,
    pub delivery_estimation_time: i32,
    pub grpc_users_address: String,
    pub grpc_orders_address: String,
    pub grpc_analytics_address: String,
}

impl Config {
    pub async fn init() -> Config {
        dotenv().ok();
        init_subscriber().await;
        info!("Initiate application config");

        let opt = Opt::from_args();
        let db_pool = establish_connection_pool(opt.database_url.clone()).await;

        let permission_policy = Policy::new().await;
        let bind_address = opt.bind_address;
        let delivery_estimation_time = opt.delivery_estimation_time;
        let grpc_users_address = opt.grpc_users_address;
        let grpc_orders_address = opt.grpc_orders_address;
        let grpc_analytics_address = opt.grpc_analytics_address;

        Config {
            db_pool,
            permission_policy,
            bind_address,
            delivery_estimation_time,
            grpc_users_address,
            grpc_orders_address,
            grpc_analytics_address,
        }
    }
}

pub struct Application {
    server: axum::Server<AddrIncoming, IntoMakeService<axum::Router>>,
}

impl Application {
    pub async fn build(config: &Config) -> Result<Self, anyhow::Error> {
        info!("Building application");
        let config = config.clone();

        let server = Server::bind(
            &config
                .bind_address
                .clone()
                .parse()
                .expect("Error parsing socket address"),
        )
        .serve(api_v1_graphql_config(config));
        Ok(Self { server })
    }

    pub async fn run_untill_stopped(self) -> Result<(), std::io::Error> {
        info!("Running application");
        // TO do : ERRORS
        self.server.await.expect("Cannot run server");
        Ok(())
    }
}

pub struct GrpcServer {
    server: Router,
}

impl GrpcServer {
    pub async fn build() -> Result<Self, anyhow::Error> {
        info!("Building gRPC Server");
        let order_service = OrdersService {};
        let server =
            tonic::transport::Server::builder().add_service(OrdersServer::new(order_service));

        Ok(Self { server })
    }

    pub async fn run_untill_stopped(self, config: Config) -> Result<(), tonic::transport::Error> {
        info!("Running gRPC Server");
        self.server
            .serve(
                config
                    .grpc_orders_address
                    .parse()
                    .expect("Cannot parse Socket Address"),
            )
            .await
    }
}
