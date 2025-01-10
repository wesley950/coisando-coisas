-- enum for user status:
-- PENDING: user has not confirmed email
-- CONFIRMED: user has confirmed email
-- DISABLED: user either terminated account or was banned
CREATE TYPE user_status AS ENUM ('PENDING', 'CONFIRMED', 'DISABLED');

CREATE TABLE users(
    id UUID PRIMARY KEY,
    nickname VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    hashed_password VARCHAR(255) NOT NULL,
    avatar_seed UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status user_status NOT NULL DEFAULT 'PENDING'
);

-- table for confirmation codes
CREATE TABLE confirmation_codes(
    user_id UUID NOT NULL UNIQUE,
    code UUID NOT NULL UNIQUE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    PRIMARY KEY (user_id, code)
);