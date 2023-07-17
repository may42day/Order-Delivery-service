use crate::schema::graphql_schema::{CourierStatus, MutationType};
use crate::utils::grpc::orders_grpc::orders_server::Orders;
use crate::utils::grpc::orders_grpc::{
    CourierForUserRequest, CourierForUserResponse, TimeExpirationRequest, TimeExpirationResponse,
};
use crate::utils::simple_broker::SimpleBroker;
use crate::{
    handlers::orders_handler,
    models::orders_model::{BucketItem, OrderInfo, OrderItem, OrderQueueInfo, ProductInfo},
    repository::orders_repository::select_bucket_items_by_uuid,
    resources::postgresql::DbConn,
    utils::{
        configs::Config,
        graphql_utils::{has_access, policy_from_context},
    },
};
use async_graphql::{Context, FieldResult, Object};
use chrono::Utc;
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

#[Object]
impl ProductInfo {
    async fn uuid(&self) -> &Uuid {
        &self.uuid
    }
    async fn name(&self) -> String {
        self.name.clone()
    }
    async fn price(&self) -> f64 {
        self.price
    }

    // Example of secured field.
    // Available only for admins and analysts
    async fn product_type(&self, context: &Context<'_>) -> FieldResult<String> {
        let policy = policy_from_context(context)?;
        if has_access(&policy.admin_policy, context) {
            Ok(self.product_type.clone())
        } else {
            Ok("Forbidden".to_string())
        }
    }

    async fn restaurant(&self) -> String {
        self.restaurant.clone()
    }
}

#[Object]
impl BucketItem {
    async fn amount(&self) -> i16 {
        self.amount
    }
    async fn product(&self, context: &Context<'_>) -> FieldResult<ProductInfo> {
        context
            .data_unchecked::<orders_handler::Products>()
            .product(context, self.product_uuid)
            .await
    }
}

#[Object]
impl OrderInfo {
    async fn uuid(&self) -> &Uuid {
        &self.uuid
    }
    async fn user_uuid(&self) -> Uuid {
        self.user_uuid
    }
    async fn courier_uuid(&self) -> Uuid {
        self.courier_uuid
    }
    async fn rating(&self) -> Option<i16> {
        self.rating
    }
    async fn status(&self) -> String {
        self.status.clone()
    }
    async fn address(&self) -> String {
        self.address.clone()
    }
    async fn items(&self, context: &Context<'_>) -> FieldResult<Vec<OrderItem>> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .get_order_items(context, self.uuid)
            .await
    }
}

#[Object]
impl OrderItem {
    async fn amount(&self) -> i16 {
        self.amount
    }
    async fn product(&self, context: &Context<'_>) -> FieldResult<ProductInfo> {
        context
            .data_unchecked::<orders_handler::Products>()
            .product(context, self.product_uuid)
            .await
    }
}

#[Object]
impl OrderQueueInfo {
    async fn status(&self) -> String {
        self.status.clone()
    }
    async fn avg_waiting_time(&self) -> i32 {
        self.avg_waiting_time
    }
}

pub async fn check_bucket(
    db_conn: &mut DbConn<'_>,
    user_uuid: Uuid,
) -> FieldResult<Vec<BucketItem>> {
    let items = select_bucket_items_by_uuid(db_conn, user_uuid).await?;
    if items.is_empty() {
        return Err("Empty bucket".into());
    }
    Ok(items)
}

pub async fn check_time_expiration(context: &Context<'_>, order: &OrderInfo) -> FieldResult<()> {
    let delivery_estimation_time = context
        .data::<Config>()
        .expect("Cannot parse AppState from context")
        .delivery_estimation_time as i64;
    let naive_date_time = Utc::now().naive_utc();
    match order.status.as_str() {
        "FINISHED" => {
            let difference = (naive_date_time - order.updated_at).num_seconds();
            if delivery_estimation_time - difference > 0 {
                Ok(())
            } else {
                Err("not available anymore".into())
            }
        }
        _ => Err("not available at the moment".into()),
    }
}

pub struct OrdersService {
    // pub db_pool: DbPool,
    // pub create_order_crone: i32,
    // pub jwt_secret: String,
}

#[tonic::async_trait]
impl Orders for OrdersService {
    async fn notify_founded_courier(
        &self,
        request: Request<CourierForUserRequest>,
    ) -> Result<Response<CourierForUserResponse>, Status> {
        let request = request.into_inner();
        let courier_uuid = request.courier_uuid;
        let _courier_rating = request.courier_rating;
        let user_uuid = Uuid::parse_str(&request.user_uuid).expect("Cannot parse string");
        println!("new courier: {:?} for user {:?}", courier_uuid, user_uuid);

        SimpleBroker::publish(CourierStatus {
            mutation_type: MutationType::Completed,
            user_uuid,
        });

        Err(Status::new(Code::Internal, "1".to_string()))
    }

    async fn notify_expiration_time(
        &self,
        request: Request<TimeExpirationRequest>,
    ) -> Result<Response<TimeExpirationResponse>, Status> {
        let user_uuid = &request.into_inner().user_uuid;
        let user_uuid = Uuid::parse_str(user_uuid).expect("Cannot parse string");
        println!("expiration time: {:?}", user_uuid);
        SimpleBroker::publish(CourierStatus {
            mutation_type: MutationType::Expired,
            user_uuid,
        });

        Err(Status::new(Code::Internal, "1".to_string()))
    }
}
