CREATE TABLE object_updates (
    id          INTEGER NOT NULL,
    full_id     TEXT PRIMARY KEY NOT NULL,
    parent      TEXT,
    pcode       TEXT,

    pos_x       REAL NOT NULL,
    pos_y       REAL NOT NULL,
    pos_z       REAL NOT NULL,

    rot_x       REAL NOT NULL,
    rot_y       REAL NOT NULL,
    rot_z       REAL NOT NULL,
    rot_w       REAL NOT NULL,

    scale_x     REAL NOT NULL,
    scale_y     REAL NOT NULL,
    scale_z     REAL NOT NULL,

    json        TEXT
);
