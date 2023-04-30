-- Add migration script here
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    name text  NOT NULL,
    email text NOT NULL unique,
    subscribed_at timestamptz NOT NULL,
    PRIMARY KEY (id)
);