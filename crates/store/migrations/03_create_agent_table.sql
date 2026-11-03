CREATE TABLE agents (
    agent_id TEXT PRIMARY KEY NOT NULL,
    local_id INGEGER,
    last_update DATETIME DEFAULT CURRENT_TIMESTAMP,
    version INTEGER,
    path TEXT
);


CREATE TABLE agent_items (
    agent_id TEXT NOT NULL REFERENCES agents(agent_id),
    path TEXT, 
    PRIMARY KEY (agent_id)
);
