-- Your SQL goes here
CREATE TABLE todos (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT false
);
