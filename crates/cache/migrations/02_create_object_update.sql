CREATE TABLE object_updates (
    id          INTEGER NOT NULL,
    full_id     TEXT PRIMARY KEY NOT NULL,
    crc         INTEGER NOT NULL,
    region_id   TEXT NOT NULL,
    parent      INTEGER,
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
    
    asset_id    TEXT,
    json        TEXT,
    glb         TEXT
);
