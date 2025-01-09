-- enum for listing status
-- DRAFT: listing is not yet published
-- PUBLISHED: listing is published and visible to the public
-- HIDDEN: listing is published but not visible to the public, only admins and creator can see
-- INACTIVE: listing is no longer visible to the public
CREATE TYPE listing_status AS ENUM ('DRAFT', 'PUBLISHED', 'HIDDEN', 'INACTIVE');

-- enum for what campus the listing is available for pickup
CREATE TYPE listing_campus AS ENUM ('DARCY', 'GAMA', 'PLANALTINA', 'CEILANDIA');

-- enum for type of listing
CREATE TYPE listing_type AS ENUM ('DONATION', 'LOAN', 'EXCHANGE', 'REQUEST');

CREATE TABLE listings (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(4096) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status listing_status NOT NULL DEFAULT 'DRAFT',
    campus listing_campus NOT NULL,
    creator_id UUID NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id)
);