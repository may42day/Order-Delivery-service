// @generated automatically by Diesel CLI.

diesel::table! {
    bucket (id) {
        id -> Int8,
        user_uuid -> Uuid,
        product_uuid -> Uuid,
        amount -> Int2,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    order_item (id) {
        id -> Int8,
        order_uuid -> Uuid,
        product_uuid -> Uuid,
        amount -> Int2,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    orders (uuid) {
        uuid -> Uuid,
        user_uuid -> Uuid,
        courier_uuid -> Uuid,
        rating -> Nullable<Int2>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    product (uuid) {
        uuid -> Uuid,
        name -> Text,
        price -> Float8,
        product_type -> Text,
        restaurant -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(order_item -> orders (order_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    bucket,
    order_item,
    orders,
    product,
);
