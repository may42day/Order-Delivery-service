ALTER TABLE orders
    DROP COLUMN address;
    
ALTER TABLE bucket
    DROP CONSTRAINT unique_product_in_user_bucket;
