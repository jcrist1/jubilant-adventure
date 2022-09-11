CREATE TABLE IF NOT EXISTS blog_posts
(
    id          BIGSERIAL PRIMARY KEY,
    title TEXT    NOT NULL,
    description TEXT    NOT NULL,
    created TIMESTAMPTZ  NOT NULL,
    updated TIMESTAMPTZ NOT NULL
);
