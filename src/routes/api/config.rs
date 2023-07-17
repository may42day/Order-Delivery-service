use crate::{
    routes::api::v1::graphql_routes::{graphiql, graphql_handler},
    utils::{configs::Config, graphql_utils::build_schema},
};
use async_graphql_axum::GraphQLSubscription;
use axum::{
    routing::{get, IntoMakeService},
    Extension, Router,
};

pub fn api_v1_graphql_config(config: Config) -> IntoMakeService<Router> {
    let schema = build_schema(config.clone());
    Router::new()
        .route("/api/v1/graphql", get(graphiql).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
        .layer(Extension(config))
        .into_make_service()
}
