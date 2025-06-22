CREATE TABLE subscriptions (
    id uuid NOT NULL DEFAULT uuid_generate_v4() PRIMARY KEY,
    email text NOT NULL UNIQUE,
    user_name text NOT NULL,
    subscribed_at timestamptz NOT NULL DEFAULT now()
);
