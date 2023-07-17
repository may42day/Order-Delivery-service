ALTER TABLE orders
    ADD COLUMN address TEXT NOT NULL;

ALTER TABLE bucket 
    ADD CONSTRAINT unique_product_in_user_bucket 
        UNIQUE (user_uuid, product_uuid);