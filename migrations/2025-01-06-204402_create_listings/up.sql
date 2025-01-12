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
    campus listing_campus NOT NULL,
    type listing_type NOT NULL,
    creator_id UUID NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id)
);

-- images attached to a listing
CREATE TABLE attachments (
    id UUID NOT NULL,
    listing_id UUID NOT NULL,
    PRIMARY KEY(id, listing_id),
    FOREIGN KEY (listing_id) REFERENCES listings(id)
);
