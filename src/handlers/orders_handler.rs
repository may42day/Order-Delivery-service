use crate::models::orders_model::{
    BucketItem, CreateOrder, OrderInfo, OrderItem, OrderQueueInfo, UpdateProduct,
};
use crate::repository::orders_repository::{
    delete_items_from_user_bucket, move_from_bucket_to_order, update_order_rating,
};
use crate::resources::postgresql::execute_connection;
use crate::services::orders_service::{check_bucket, check_time_expiration};
use crate::services::users_service::{
    check_courier_from_queue, find_free_courier, update_courier_rating,
};
use crate::utils::graphql_utils::{
    has_access, has_access_by_uuid, has_access_to_filters, has_access_to_order, policy_from_context,
};
use crate::{
    models::orders_model::{CreateProduct, ProductInfo},
    repository::orders_repository,
    schema::graphql_schema::{MutationRoot, QueryRoot, SubscriptionRoot},
};

use async_graphql::{Context, FieldResult, Schema};
use uuid::Uuid;

pub type OrderServiceSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
pub struct Products;
pub struct Buckets;
pub struct Orders;

impl Products {
    pub async fn product(&self, context: &Context<'_>, uuid: Uuid) -> FieldResult<ProductInfo> {
        let mut db_conn = execute_connection(context).await?;

        let product = orders_repository::select_product(&mut db_conn, uuid).await?;
        Ok(product)
    }

    pub async fn products(
        &self,
        context: &Context<'_>,
        name: Option<String>,
        price_from_cheap: Option<bool>,
        price_from_expensive: Option<bool>,
        product_type: Option<String>,
        restaurant: Option<String>,
    ) -> FieldResult<Vec<ProductInfo>> {
        let mut db_conn = execute_connection(context).await?;

        let products = orders_repository::select_products_by_filter(
            &mut db_conn,
            name,
            price_from_cheap,
            price_from_expensive,
            product_type,
            restaurant,
        )
        .await?;
        Ok(products)
    }

    pub async fn update_product(
        &self,
        context: &Context<'_>,
        name: Option<String>,
        price: Option<f64>,
        product_type: Option<String>,
        restaurant: Option<String>,
        uuid: Uuid,
    ) -> FieldResult<ProductInfo> {
        let policy = policy_from_context(context)?;
        if !has_access(&policy.admin_policy, context) {
            return Err("Forbidden".into());
        };
        let mut db_conn = execute_connection(context).await?;

        let product = UpdateProduct {
            name,
            price,
            product_type,
            restaurant,
        };
        let product = orders_repository::update_product(&mut db_conn, product, uuid).await?;
        Ok(product)
    }

    pub async fn create_product(
        &self,
        context: &Context<'_>,
        name: String,
        price: f64,
        product_type: String,
        restaurant: String,
    ) -> FieldResult<String> {
        let policy = policy_from_context(context)?;
        if !has_access(&policy.admin_policy, context) {
            return Err("Forbidden".into());
        };
        let mut db_conn = execute_connection(context).await?;

        let product = CreateProduct {
            name,
            price,
            product_type,
            restaurant,
        };
        orders_repository::create_product(&mut db_conn, product).await?;

        Ok("Product created".into())
    }
}

impl Buckets {
    pub async fn add_to_bucket(
        &self,
        context: &Context<'_>,
        user_uuid: Uuid,
        product_uuid: Uuid,
        amount: i16,
    ) -> FieldResult<BucketItem> {
        // let policy = policy_from_context(context)?;
        // if !has_access(&policy.admin_policy, context) {
        //     println!("don't have access");
        //     return Err("Forbidden".into())
        // };
        has_access_by_uuid(context, user_uuid).await?;
        let mut db_conn = execute_connection(context).await?;

        let bucket_item = BucketItem {
            user_uuid,
            product_uuid,
            amount,
        };
        let bucket_item = orders_repository::create_bucket_item(&mut db_conn, bucket_item).await?;
        Ok(bucket_item)
    }

    pub async fn get_bucket_items(
        &self,
        context: &Context<'_>,
        uuid: Uuid,
    ) -> FieldResult<Vec<BucketItem>> {
        let policy = policy_from_context(context)?;
        if !has_access(&policy.admin_policy, context) {
            has_access_by_uuid(context, uuid).await?;
        };

        let mut db_conn = execute_connection(context).await?;
        let items = orders_repository::select_bucket_items_by_uuid(&mut db_conn, uuid).await?;
        Ok(items)
    }

    pub async fn remove_from_bucket(
        &self,
        context: &Context<'_>,
        user_uuid: Uuid,
        product_uuid: Uuid,
    ) -> FieldResult<String> {
        // let policy = policy_from_context(context)?;
        // if !has_access(&policy.admin_policy, context) {
        //     return Err("Forbidden".into())
        // };
        has_access_by_uuid(context, user_uuid).await?;
        let mut db_conn = execute_connection(context).await?;

        orders_repository::delete_item_from_bucket(&mut db_conn, user_uuid, product_uuid).await?;
        Ok("Item deleted".to_string())
    }

    pub async fn clear_bucket(
        &self,
        context: &Context<'_>,
        user_uuid: Uuid,
    ) -> FieldResult<String> {
        // let policy = policy_from_context(context)?;
        // if !has_access(&policy.admin_policy, context) {
        //     return Err("Forbidden".into())
        // };
        has_access_by_uuid(context, user_uuid).await?;
        let mut db_conn = execute_connection(context).await?;

        orders_repository::delete_items_from_user_bucket(&mut db_conn, user_uuid).await?;
        Ok("Bucket cleared".to_string())
    }
}

impl Orders {
    pub async fn create_order(
        &self,
        context: &Context<'_>,
        user_uuid: Uuid,
        address: String,
    ) -> FieldResult<OrderInfo> {
        has_access_by_uuid(context, user_uuid).await?;
        let mut db_conn = execute_connection(context).await?;

        println!("checking bucket");
        let bucket = check_bucket(&mut db_conn, user_uuid).await?;
        println!("bucket checked");
        println!("searching for new courier");
        // searching for a free courier or adding user in queue
        // if there are no free couriers
        let courier_uuid = find_free_courier(context, user_uuid).await?;
        println!("couriers finded");
        let order = CreateOrder {
            user_uuid,
            courier_uuid,
            address,
        };
        println!("creating order");
        let order = orders_repository::create_order(&mut db_conn, order).await?;
        println!("order created");
        let order_items = bucket
            .iter()
            .map(|item| OrderItem {
                order_uuid: order.uuid,
                product_uuid: item.product_uuid,
                amount: item.amount,
            })
            .collect::<Vec<OrderItem>>();
        println!("moving items from bucket to order");
        move_from_bucket_to_order(&mut db_conn, order_items).await?;
        println!("items from bucket moved");
        println!("deleting items from bucket");
        delete_items_from_user_bucket(&mut db_conn, user_uuid).await?;
        println!("items from bucket deleted");

        Ok(order)
    }

    pub async fn order(&self, context: &Context<'_>, uuid: Uuid) -> FieldResult<OrderInfo> {
        let mut db_conn = execute_connection(context).await?;

        let order = orders_repository::select_order(&mut db_conn, uuid).await?;
        let policy = policy_from_context(context)?;
        has_access_to_order(&policy.analyst_policy, context, order.clone()).await?;
        Ok(order)
    }

    // Filter orders by optional filetrs
    // All orders available for admins and analysts
    // Users and couriers has access only for their own orders
    // user_uuid required for Users
    // courier_uuid required for Couriers
    pub async fn filter_orders(
        &self,
        context: &Context<'_>,
        order_uuid: Option<Uuid>,
        courier_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        address: Option<String>,
    ) -> FieldResult<Vec<OrderInfo>> {
        has_access_to_filters(context, courier_uuid, user_uuid).await?;
        let mut db_conn = execute_connection(context).await?;
        let orders = orders_repository::select_orders_by_filters(
            &mut db_conn,
            order_uuid,
            courier_uuid,
            user_uuid,
            address,
        )
        .await?;
        Ok(orders)
    }

    pub async fn get_order_items(
        &self,
        context: &Context<'_>,
        uuid: Uuid,
    ) -> FieldResult<Vec<OrderItem>> {
        let mut db_conn = execute_connection(context).await?;
        let order = orders_repository::select_order(&mut db_conn, uuid).await?;

        let policy = policy_from_context(context)?;
        has_access_to_order(&policy.admin_policy, context, order).await?;
        let items = orders_repository::select_order_items_by_uuid(&mut db_conn, uuid).await?;
        Ok(items)
    }

    pub async fn estimate_delivery(
        &self,
        context: &Context<'_>,
        order_uuid: Uuid,
        rating: i16,
    ) -> FieldResult<String> {
        let mut db_conn = execute_connection(context).await?;
        let order = orders_repository::select_order(&mut db_conn, order_uuid).await?;
        match order.rating {
            Some(_) => Err("This order already rated".into()),
            None => {
                has_access_by_uuid(context, order.user_uuid).await?;
                check_time_expiration(context, &order).await?;
                update_order_rating(&mut db_conn, order.uuid, rating).await?;
                update_courier_rating(&mut db_conn, context, order.courier_uuid).await
            }
        }
    }

    pub async fn complete_delivery(
        &self,
        context: &Context<'_>,
        order_uuid: Uuid,
    ) -> FieldResult<String> {
        let mut db_conn = execute_connection(context).await?;
        let order = orders_repository::select_order(&mut db_conn, order_uuid).await?;
        match order.status.as_str() {
            "IN_PROGRESS" => {
                has_access_by_uuid(context, order.courier_uuid).await?;
                let new_status = "FINISHED";
                orders_repository::update_order_status(&mut db_conn, order_uuid, new_status)
                    .await?;
                Ok("Delivery finished".to_string())
            }
            _ => Err("This order already finished".into()),
        }
    }

    pub async fn wait_for_free_courier(
        &self,
        context: &Context<'_>,
        user_uuid: Uuid,
    ) -> FieldResult<OrderQueueInfo> {
        has_access_by_uuid(context, user_uuid).await?;
        check_courier_from_queue(context, user_uuid).await
    }
}
