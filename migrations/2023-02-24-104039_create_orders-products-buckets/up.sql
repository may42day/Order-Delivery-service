CREATE EXTENSION "uuid-ossp";

CREATE TABLE product (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    price FLOAT NOT NULL,
    product_type TEXT NOT NULL,
    restaurant TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE bucket (
    id BIGSERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL,
    product_uuid UUID NOT NULL,
    amount smallint NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT FK_PRODUCT
        FOREIGN KEY(product_uuid)
            REFERENCES product(uuid)
);

CREATE INDEX idx_bucket ON bucket (user_uuid);

CREATE TABLE orders (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_uuid UUID NOT NULL,
    courier_uuid UUID NOT NULL,
    rating smallint,
    status TEXT NOT NULL DEFAULT 'IN_PROGRESS',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT USERS_ROLE_CHECK
        CHECK (status in ('IN_PROGRESS', 'FINISHED', 'CANCELED'))
);

CREATE TABLE order_item(
    id BIGSERIAL PRIMARY KEY,
    order_uuid UUID NOT NULL,
    product_uuid UUID NOT NULL,
    amount smallint NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT FK_PRODUCT
        FOREIGN KEY(product_uuid)
            REFERENCES product(uuid),
    CONSTRAINT FK_ORDER
        FOREIGN KEY(order_uuid)
            REFERENCES orders(uuid)
);

CREATE INDEX idx_order_item ON order_item (order_uuid);

CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_timestamp_product
BEFORE UPDATE ON product
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

CREATE TRIGGER set_timestamp_bucket
BEFORE UPDATE ON bucket
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

CREATE TRIGGER set_timestamp_orders
BEFORE UPDATE ON orders
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

CREATE TRIGGER set_timestamp_order_item
BEFORE UPDATE ON order_item
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();