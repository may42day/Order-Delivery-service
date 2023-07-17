use crate::schema::diesel_schema::{bucket, order_item, orders, product};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Order {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub courier_uuid: Uuid,
    pub rating: Option<i16>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub address: String,
}

#[derive(Insertable)]
#[diesel(table_name = orders)]
pub struct CreateOrder {
    pub user_uuid: Uuid,
    pub courier_uuid: Uuid,
    pub address: String,
}

#[derive(Queryable)]
#[diesel(table_name = order_item)]
pub struct OrderItems {
    pub id: i64,
    pub order_uuid: Uuid,
    pub product_uuid: Uuid,
    pub amount: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = order_item)]
pub struct OrderItem {
    pub order_uuid: Uuid,
    pub product_uuid: Uuid,
    pub amount: i16,
}

#[derive(Queryable)]
pub struct Bucket {
    pub id: i64,
    pub user_uuid: Uuid,
    pub product_uuid: Uuid,
    pub amount: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable)]
pub struct Product {
    pub uuid: Uuid,
    pub name: String,
    pub price: f64,
    pub product_type: String,
    pub restaurant: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = product)]
pub struct CreateProduct {
    pub name: String,
    pub price: f64,
    pub product_type: String,
    pub restaurant: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = product)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub price: Option<f64>,
    pub product_type: Option<String>,
    pub restaurant: Option<String>,
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = product)]
pub struct ProductInfo {
    pub uuid: Uuid,
    pub name: String,
    pub price: f64,
    pub product_type: String,
    pub restaurant: String,
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = bucket)]
pub struct BucketItem {
    pub user_uuid: Uuid,
    pub product_uuid: Uuid,
    pub amount: i16,
}

#[derive(Queryable, Clone)]
pub struct OrderInfo {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub courier_uuid: Uuid,
    pub rating: Option<i16>,
    pub status: String,
    pub updated_at: NaiveDateTime,
    pub address: String,
}

#[derive(Clone)]
pub struct OrderQueueInfo {
    pub status: String,
    pub avg_waiting_time: i32,
}
