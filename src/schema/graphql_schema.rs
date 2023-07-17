use crate::handlers::orders_handler;
use crate::models::orders_model::{BucketItem, OrderInfo, OrderItem, OrderQueueInfo, ProductInfo};
use crate::utils::simple_broker::SimpleBroker;
use async_graphql::futures_util::Stream;
use async_graphql::{Context, FieldResult, MergedObject, Object, Subscription};
use futures_util::StreamExt;
use uuid::Uuid;

#[derive(MergedObject, Default)]
pub struct QueryRoot(Products, Bucket, Orders);

#[derive(MergedObject, Default)]
pub struct MutationRoot(ProductsMutation, BucketMutation, OrdersMutation);

#[derive(Default)]
pub struct Products;

#[derive(Default)]
pub struct Orders;

#[derive(Default)]
pub struct Bucket;

#[derive(Default)]
pub struct ProductsMutation;

#[derive(Default)]
pub struct OrdersMutation;

#[derive(Default)]
pub struct BucketMutation;

#[derive(Default)]
pub struct SubscriptionRoot;

#[Object]
impl Products {
    // Get product by uuid
    // "uuid" required
    pub async fn product<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "uuid of product")] uuid: Uuid,
    ) -> FieldResult<ProductInfo> {
        context
            .data_unchecked::<orders_handler::Products>()
            .product(context, uuid)
            .await
    }

    // Get all products
    pub async fn products<'a>(
        &self,
        context: &Context<'a>,
        name: Option<String>,
        price_from_cheap: Option<bool>,
        price_from_expensive: Option<bool>,
        product_type: Option<String>,
        restaurant: Option<String>,
    ) -> FieldResult<Vec<ProductInfo>> {
        context
            .data_unchecked::<orders_handler::Products>()
            .products(
                context,
                name,
                price_from_cheap,
                price_from_expensive,
                product_type,
                restaurant,
            )
            .await
    }
}

#[Object]
impl ProductsMutation {
    // Creating new product
    // "name", "price", "product type", "restaurant" required
    pub async fn create_product<'a>(
        &self,
        context: &Context<'a>,
        name: String,
        price: f64,
        product_type: String,
        restaurant: String,
    ) -> FieldResult<String> {
        context
            .data_unchecked::<orders_handler::Products>()
            .create_product(context, name, price, product_type, restaurant)
            .await
    }

    // Updating product info
    // "uuid" required
    // "name", "price", " product type", "restaurant" optional
    pub async fn update_product<'a>(
        &self,
        context: &Context<'a>,
        name: Option<String>,
        price: Option<f64>,
        product_type: Option<String>,
        restaurant: Option<String>,
        #[graphql(desc = "uuid of product")] uuid: Uuid,
    ) -> FieldResult<ProductInfo> {
        context
            .data_unchecked::<orders_handler::Products>()
            .update_product(context, name, price, product_type, restaurant, uuid)
            .await
    }
}
#[Object]
impl Orders {
    // Get order info
    // "uuid" required
    pub async fn order<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "uuid of order")] uuid: Uuid,
    ) -> FieldResult<OrderInfo> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .order(context, uuid)
            .await
    }

    // Get orders info with filters
    // optional filters: "order_uuid", "courier_uuid", "user_uuid", "address"
    // geting all orders if no filters provided
    pub async fn filter_orders<'a>(
        &self,
        context: &Context<'a>,
        order_uuid: Option<Uuid>,
        courier_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        address: Option<String>,
    ) -> FieldResult<Vec<OrderInfo>> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .filter_orders(context, order_uuid, courier_uuid, user_uuid, address)
            .await
    }

    // Get orders items info
    // "uuid" required
    // Show all products with its amount in order
    pub async fn order_items<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "uuid of order")] uuid: Uuid,
    ) -> FieldResult<Vec<OrderItem>> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .get_order_items(context, uuid)
            .await
    }
}

#[Object]
impl OrdersMutation {
    // Creating new order
    // "user_uuid", "address" required
    pub async fn create_order<'a>(
        &self,
        context: &Context<'a>,
        user_uuid: Uuid,
        address: String,
    ) -> FieldResult<OrderInfo> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .create_order(context, user_uuid, address)
            .await
    }

    pub async fn estimate_delivery<'a>(
        &self,
        context: &Context<'a>,
        order_uuid: Uuid,
        rating: i16,
    ) -> FieldResult<String> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .estimate_delivery(context, order_uuid, rating)
            .await
    }

    pub async fn complete_delivery<'a>(
        &self,
        context: &Context<'_>,
        order_uuid: Uuid,
    ) -> FieldResult<String> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .complete_delivery(context, order_uuid)
            .await
    }
}

#[Object]
impl Bucket {
    // Get items from user bucket
    // "uuid" required
    // Show all products with its amount in user bucket
    pub async fn bucket_items<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "user uuid of bucket")] uuid: Uuid,
    ) -> FieldResult<Vec<BucketItem>> {
        context
            .data_unchecked::<orders_handler::Buckets>()
            .get_bucket_items(context, uuid)
            .await
    }
}

#[Object]
impl BucketMutation {
    // Add product with qty to user's bucket
    // "user_uuid", "product_uuid", "amount" required
    pub async fn add_to_bucket<'a>(
        &self,
        context: &Context<'a>,
        user_uuid: Uuid,
        product_uuid: Uuid,
        amount: i16,
    ) -> FieldResult<BucketItem> {
        context
            .data_unchecked::<orders_handler::Buckets>()
            .add_to_bucket(context, user_uuid, product_uuid, amount)
            .await
    }

    // Remove product from user's bucket
    // "user_uuid", "product_uuid" required
    pub async fn remove_from_bucket<'a>(
        &self,
        context: &Context<'a>,
        user_uuid: Uuid,
        product_uuid: Uuid,
    ) -> FieldResult<String> {
        context
            .data_unchecked::<orders_handler::Buckets>()
            .remove_from_bucket(context, user_uuid, product_uuid)
            .await
    }

    // Clear user's bucket
    // "user_uuid" required
    pub async fn clear_bucket<'a>(
        &self,
        context: &Context<'a>,
        user_uuid: Uuid,
    ) -> FieldResult<String> {
        context
            .data_unchecked::<orders_handler::Buckets>()
            .clear_bucket(context, user_uuid)
            .await
    }

    pub async fn wait_for_free_courier<'a>(
        &self,
        context: &Context<'a>,
        user_uuid: Uuid,
    ) -> FieldResult<OrderQueueInfo> {
        context
            .data_unchecked::<orders_handler::Orders>()
            .wait_for_free_courier(context, user_uuid)
            .await
    }
}

/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////
#[Subscription]
impl SubscriptionRoot {
    async fn interval(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        async_graphql::async_stream::stream! {
            loop {
                futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
                value += n;
                yield value;
            }
        }
    }

    async fn orders(&self, user_uuid: Uuid) -> impl Stream<Item = CourierStatus> {
        SimpleBroker::<CourierStatus>::subscribe().filter(move |event| {
            // let res = if let Some(mutation_type) = mutation_type {
            //     event.mutation_type == mutation_type
            // } else {
            //     true
            // };
            // async move { res }

            // let res = if event.user_uuid == user_uuid {
            //     true
            // } else {
            //     false
            // };
            // async move { res }

            let res = event.user_uuid == user_uuid;
            async move { res }
        })
    }
}

use async_graphql::{Enum, Result, ID};
use futures_util::lock::Mutex;
use slab::Slab;

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
pub enum MutationType {
    Created,
    Deleted,

    Completed,
    Expired,
}

#[derive(Clone)]
pub struct CourierStatus {
    pub mutation_type: MutationType,
    pub user_uuid: Uuid,
}

#[Object]
impl CourierStatus {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn id(&self) -> &Uuid {
        &self.user_uuid
    }

    // async fn order(&self, ctx: &Context<'_>) -> Result<Option<Book>> {
    //     let books = ctx.data_unchecked::<Storage>().lock().await;
    //     let id = self.id.parse::<usize>()?;
    //     Ok(books.get(id).cloned())
    // }
}

// #[derive(Clone)]
// pub struct Book {
//     id: ID,
//     name: String,
//     author: String,
// }

// #[Object]
// impl Book {
//     async fn id(&self) -> &str {
//         &self.id
//     }

//     async fn name(&self) -> &str {
//         &self.name
//     }

//     async fn author(&self) -> &str {
//         &self.author
//     }
// }

// pub type Storage = Arc<Mutex<Slab<Book>>>;
