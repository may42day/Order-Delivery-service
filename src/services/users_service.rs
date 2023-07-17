use crate::{
    models::orders_model::OrderQueueInfo,
    repository::orders_repository::get_courier_rating,
    resources::postgresql::DbConn,
    utils::{
        configs::Config,
        grpc::users_grpc::{
            users_client::UsersClient, FindCourierRequest, TokenClaimsRequest,
            UpdateCourierRatingRequest, WaitForCourierRequest,
        },
    },
};
use async_graphql::{Context, FieldResult};
use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization, HeaderMapExt},
    http::{self, request::Parts},
    Extension, RequestPartsExt,
};
use hyper::StatusCode;
use tonic::{Code, Status};
use uuid::Uuid;

pub async fn get_token_claims(
    token: String,
    grpc_users_address: String,
) -> Result<TokenClaims, Status> {
    let mut client = UsersClient::connect(grpc_users_address)
        .await
        .expect("Cannot connect to user service");
    let request = tonic::Request::new(TokenClaimsRequest { token });

    let response = client.send_token_claims(request).await;
    match response {
        Ok(response) => {
            let token_claims = response.into_inner();
            Ok(TokenClaims {
                uuid: Uuid::parse_str(&token_claims.uuid).expect("Cannot parse uuid from string"),
                role: token_claims.role,
            })
        }
        Err(status) => Err(status),
    }
}

#[derive(Default, Debug)]
pub struct TokenClaims {
    pub uuid: Uuid,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for TokenClaims
where
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .typed_get::<Authorization<Bearer>>()
            .ok_or(StatusCode::UNAUTHORIZED)?
            .token()
            .to_owned();

        let Extension(config) = parts
            .extract::<Extension<Config>>()
            .await
            .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

        let token_claims = get_token_claims(token, config.grpc_users_address).await;
        match token_claims {
            Ok(claims) => Ok(claims),
            Err(status) => match status.code() {
                Code::Unauthenticated => Err(StatusCode::UNAUTHORIZED),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            },
        }
    }
}

pub async fn find_free_courier(context: &Context<'_>, user_uuid: Uuid) -> FieldResult<Uuid> {
    let config = &context
        .data::<Config>()
        .expect("Cannot parse AppState from context");
    let mut client = UsersClient::connect(config.grpc_users_address.clone())
        .await
        .expect("Cannot connect to user service");

    let request = tonic::Request::new(FindCourierRequest {
        user_uuid: user_uuid.to_string(),
    });
    println!("--find_free_courier making request");
    let response = client.find_courier(request).await;
    println!("--find_free_courier handling response");
    match response {
        Ok(response) => {
            let response = response.into_inner();
            if response.added_to_queue {
                // In case user was added in queue
                Err("Added to queue".into())
            } else {
                Ok(Uuid::parse_str(&response.courier_uuid).expect("Cannot parse uuid"))
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR.into()),
    }
}

pub async fn update_courier_rating(
    db_conn: &mut DbConn<'_>,
    context: &Context<'_>,
    courier_uuid: Uuid,
) -> FieldResult<String> {
    let rating = count_average_rating(db_conn, courier_uuid).await?;
    let config = &context
        .data::<Config>()
        .expect("Cannot parse AppState from context");
    let mut client = UsersClient::connect(config.grpc_users_address.clone())
        .await
        .expect("Cannot connect to user service");

    let request = tonic::Request::new(UpdateCourierRatingRequest {
        courier_uuid: courier_uuid.to_string(),
        rating,
    });
    let result = client.update_courier_rating(request).await;
    match result {
        Ok(result) => Ok(result.into_inner().message),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR.into()),
    }
}

pub async fn count_average_rating(
    db_conn: &mut DbConn<'_>,
    courier_uuid: Uuid,
) -> FieldResult<f32> {
    let courier_rating = get_courier_rating(db_conn, courier_uuid).await?;
    let mut rating: Vec<i16> = courier_rating.into_iter().map(|x| x.unwrap()).collect();
    let mut length = rating.len();
    match length {
        0..=99 => {
            let additional_rating = [5; 50];
            rating.extend_from_slice(&additional_rating);
        }
        100..=149 => {
            rating.extend(vec![1; 150 - length]);
        }
        _ => (),
    }

    length = rating.len();
    let weight_sum = ((1 + length) as f32 / 2.0) * length as f32;
    let mut counter = (length + 1) as i32;
    let multiplication_weight_sum: i32 = rating
        .iter()
        .map(|x| {
            counter -= 1;
            *x as i32 * counter
        })
        .sum();
    let rating = (multiplication_weight_sum as f32) / weight_sum;
    Ok(rating)
}

pub async fn check_courier_from_queue(
    context: &Context<'_>,
    order_uuid: Uuid,
) -> FieldResult<OrderQueueInfo> {
    let config = &context
        .data::<Config>()
        .expect("Cannot parse AppState from context");
    let mut client = UsersClient::connect(config.grpc_users_address.clone())
        .await
        .expect("Cannot connect to user service");

    let request = tonic::Request::new(WaitForCourierRequest {
        order_uuid: order_uuid.to_string(),
    });
    let result = client.wait_for_courier(request).await;
    match result {
        Ok(result) => {
            let queue_info = result.into_inner();
            Ok(OrderQueueInfo {
                status: queue_info.status,
                avg_waiting_time: queue_info.avg_waiting_time,
            })
        }
        // add Access Error
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR.into()),
    }
}
