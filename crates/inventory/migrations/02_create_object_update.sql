CREATE TABLE object_updates (
    id                  INTEGER NOT NULL,
    full_id             TEXT PRIMARY KEY NOT NULL,
    parent              TEXT,
    pcode               TEXT, 
    json                TEXT
);
