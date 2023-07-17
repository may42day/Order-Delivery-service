use crate::models::orders_model::OrderInfo;
use crate::services::users_service::TokenClaims;
use crate::{
    handlers::orders_handler::{Buckets, Orders, Products},
    schema::graphql_schema::{MutationRoot, QueryRoot, SubscriptionRoot},
};
use async_graphql::{Context, Error, Schema};
use tracing::info;
use uuid::Uuid;

use super::configs::Config;
use super::permission_policy::Policy;

pub fn build_schema(config: Config) -> Schema<QueryRoot, MutationRoot, SubscriptionRoot> {
    info!("Building GraphQL schema");
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(Products)
    .data(Buckets)
    .data(Orders)
    .data(config)
    .limit_depth(5)
    .finish()
}

// Get claims from GraphQL context
// Should not panic because of adding claims to context in GraphQL endpoint
pub fn token_claims_from_context<'a>(context: &'a Context<'a>) -> &'a TokenClaims {
    context
        .data::<TokenClaims>()
        .expect("Cannot parse TokenClaims from context")
}

pub fn policy_from_context<'a>(context: &'a Context<'a>) -> Result<&'a Policy, Error> {
    Ok(&context
        .data::<Config>()?
        .permission_policy)
}

pub fn has_access(permissions: &[String], context: &Context<'_>) -> bool {
    let claims = token_claims_from_context(context);
    permissions.contains(&claims.role)
}

pub async fn has_access_to_order(
    permissions: &[String],
    context: &Context<'_>,
    order: OrderInfo,
) -> Result<(), Error> {
    let claims = token_claims_from_context(context);
    let uuid = claims.uuid;
    let order_user_uuid = order.user_uuid;
    let order_courier_uuid = order.courier_uuid;

    if has_access(permissions, context) || uuid == order_user_uuid || uuid == order_courier_uuid {
        Ok(())
    } else {
        Err("Forbidden".into())
    }
}

pub async fn has_access_by_uuid(context: &Context<'_>, user_uuid: Uuid) -> Result<(), Error> {
    let claims = token_claims_from_context(context);
    if user_uuid == claims.uuid {
        Ok(())
    } else {
        Err("Forbidden".into())
    }
}

pub async fn has_access_to_filters(
    context: &Context<'_>,
    courier_uuid: Option<Uuid>,
    user_uuid: Option<Uuid>,
) -> Result<(), Error> {
    let claims = token_claims_from_context(context);
    let policy = policy_from_context(context)?;
    if has_access(&policy.analyst_policy, context)
        || (courier_uuid == Some(claims.uuid) && has_access(&policy.courier_policy, context))
        || user_uuid == Some(claims.uuid)
    {
        Ok(())
    } else {
        Err("Forbidden".into())
    }
}
