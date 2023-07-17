use crate::models::orders_model::*;
use crate::resources::postgresql::DbConn;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn select_products_by_filter(
    db_conn: &mut DbConn<'_>,
    pr_name: Option<String>,
    price_from_cheap: Option<bool>,
    price_from_expensive: Option<bool>,
    pr_type: Option<String>,
    pr_restaurant: Option<String>,
) -> Result<Vec<ProductInfo>, Error> {
    use crate::schema::diesel_schema::product::dsl::*;
    let mut query = product.into_boxed();

    if let Some(pr_name) = pr_name {
        query = query.order(name.like(pr_name));
    }

    if let Some(true) = price_from_cheap {
        query = query.order(price.asc());
    }

    if let Some(true) = price_from_expensive {
        query = query.order(price.desc());
    }

    if let Some(pr_type) = pr_type {
        query = query.filter(product_type.eq(pr_type));
    }

    if let Some(pr_restaurant) = pr_restaurant {
        query = query.filter(restaurant.eq(pr_restaurant));
    }

    query
        .select((uuid, name, price, product_type, restaurant))
        .get_results(db_conn)
        .await
}

pub async fn select_product(
    db_conn: &mut DbConn<'_>,
    product_uuid: Uuid,
) -> Result<ProductInfo, Error> {
    use crate::schema::diesel_schema::product::dsl::*;
    product
        .filter(uuid.eq(product_uuid))
        .select((uuid, name, price, product_type, restaurant))
        .get_result(db_conn)
        .await
}

pub async fn create_product(
    db_conn: &mut DbConn<'_>,
    new_product: CreateProduct,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::product::dsl::*;
    diesel::insert_into(product)
        .values(new_product)
        .execute(db_conn)
        .await
}

pub async fn update_product(
    db_conn: &mut DbConn<'_>,
    new_product: UpdateProduct,
    product_uuid: Uuid,
) -> Result<ProductInfo, Error> {
    use crate::schema::diesel_schema::product::dsl::*;
    diesel::update(product.find(product_uuid))
        .set(new_product)
        .returning((uuid, name, price, product_type, restaurant))
        .get_result(db_conn)
        .await
}

pub async fn create_bucket_item(
    db_conn: &mut DbConn<'_>,
    bucket_item: BucketItem,
) -> Result<BucketItem, Error> {
    use crate::schema::diesel_schema::bucket::dsl::*;
    diesel::insert_into(bucket)
        .values(bucket_item)
        .returning((user_uuid, product_uuid, amount))
        .get_result(db_conn)
        .await
}

pub async fn select_bucket_items_by_uuid(
    db_conn: &mut DbConn<'_>,
    uuid: Uuid,
) -> Result<Vec<BucketItem>, Error> {
    use crate::schema::diesel_schema::bucket::dsl::*;
    bucket
        .filter(user_uuid.eq(uuid))
        .select((user_uuid, product_uuid, amount))
        .get_results(db_conn)
        .await
}

pub async fn delete_item_from_bucket(
    db_conn: &mut DbConn<'_>,
    us_uuid: Uuid,
    prod_uuid: Uuid,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::bucket::dsl::*;
    diesel::delete(bucket.filter(user_uuid.eq(us_uuid).and(product_uuid.eq(prod_uuid))))
        .execute(db_conn)
        .await
}

pub async fn delete_items_from_user_bucket(
    db_conn: &mut DbConn<'_>,
    us_uuid: Uuid,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::bucket::dsl::*;
    diesel::delete(bucket.filter(user_uuid.eq(us_uuid)))
        .execute(db_conn)
        .await
}

pub async fn create_order(
    db_conn: &mut DbConn<'_>,
    new_order: CreateOrder,
) -> Result<OrderInfo, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    diesel::insert_into(orders)
        .values(new_order)
        .returning((
            uuid,
            user_uuid,
            courier_uuid,
            rating,
            status,
            updated_at,
            address,
        ))
        .get_result(db_conn)
        .await
}

pub async fn move_from_bucket_to_order(
    db_conn: &mut DbConn<'_>,
    items: Vec<OrderItem>,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::order_item::dsl::*;
    diesel::insert_into(order_item)
        .values(items)
        .execute(db_conn)
        .await
}

pub async fn select_order(db_conn: &mut DbConn<'_>, order_uuid: Uuid) -> Result<OrderInfo, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    orders
        .filter(uuid.eq(order_uuid))
        .select((
            uuid,
            user_uuid,
            courier_uuid,
            rating,
            status,
            updated_at,
            address,
        ))
        .get_result(db_conn)
        .await
}

pub async fn update_order_rating(
    db_conn: &mut DbConn<'_>,
    order_uuid: Uuid,
    new_rating: i16,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    diesel::update(orders)
        .filter(uuid.eq(order_uuid))
        .set(rating.eq(new_rating))
        .execute(db_conn)
        .await
}
pub async fn update_order_status(
    db_conn: &mut DbConn<'_>,
    order_uuid: Uuid,
    new_status: &str,
) -> Result<usize, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    diesel::update(orders)
        .filter(uuid.eq(order_uuid))
        .set(status.eq(new_status))
        .execute(db_conn)
        .await
}
pub async fn get_courier_rating(
    db_conn: &mut DbConn<'_>,
    c_uuid: Uuid,
) -> Result<Vec<Option<i16>>, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    orders
        .filter(courier_uuid.eq(c_uuid))
        .filter(status.eq("FINISHED".to_string()))
        .filter(rating.gt(0))
        .order(updated_at.desc())
        .limit(149)
        .select(rating)
        .get_results(db_conn)
        .await
}

pub async fn select_orders_by_filters(
    db_conn: &mut DbConn<'_>,
    order_uuid: Option<Uuid>,
    uuid_courier: Option<Uuid>,
    uuid_user: Option<Uuid>,
    order_address: Option<String>,
) -> Result<Vec<OrderInfo>, Error> {
    use crate::schema::diesel_schema::orders::dsl::*;
    let mut query = orders.into_boxed();

    if let Some(order_uuid) = order_uuid {
        query = query.filter(uuid.eq(order_uuid));
    }

    if let Some(uuid_courier) = uuid_courier {
        query = query.filter(courier_uuid.eq(uuid_courier));
    }

    if let Some(uuid_user) = uuid_user {
        query = query.filter(user_uuid.eq(uuid_user));
    }

    if let Some(order_address) = order_address {
        query = query.filter(address.eq(order_address));
    }
    query
        .select((
            uuid,
            user_uuid,
            courier_uuid,
            rating,
            status,
            updated_at,
            address,
        ))
        .get_results(db_conn)
        .await
}

pub async fn select_order_items_by_uuid(
    db_conn: &mut DbConn<'_>,
    uuid: Uuid,
) -> Result<Vec<OrderItem>, Error> {
    use crate::schema::diesel_schema::order_item::dsl::*;
    order_item
        .filter(order_uuid.eq(uuid))
        .select((order_uuid, product_uuid, amount))
        .get_results(db_conn)
        .await
}
