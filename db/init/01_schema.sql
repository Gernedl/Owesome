CREATE TABLE users (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE tags (
    id   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE transactions (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    creditor_id BIGINT NOT NULL REFERENCES users (id),
    debtor_id   BIGINT NOT NULL REFERENCES users (id),
    tag_id      BIGINT REFERENCES tags (id) ON DELETE SET NULL,
    CHECK (creditor_id <> debtor_id)
);

CREATE INDEX ON transactions (creditor_id);
CREATE INDEX ON transactions (debtor_id);
CREATE INDEX ON transactions (tag_id);
