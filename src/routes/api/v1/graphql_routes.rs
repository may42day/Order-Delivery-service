use crate::handlers::orders_handler::OrderServiceSchema;
use crate::services::users_service::TokenClaims;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::debug_handler;
use axum::extract::Extension;
use axum::response::{self, IntoResponse};

pub async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/api/v1/graphql")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

#[debug_handler]
pub async fn graphql_handler(
    schema: Extension<OrderServiceSchema>,
    token_claims: TokenClaims,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(token_claims))
        .await
        .into()
}
