CREATE TABLE agents (
    agent_id TEXT PRIMARY KEY NOT NULL, 
    local_id INTEGER, 
    position_x INTEGER, 
    position_y INTEGER, 
    position_z INTEGER, 
    skeleton TEXT, 
    path TEXT, 
    last_update DATETIME DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE agent_items (
    agent_id TEXT NOT NULL REFERENCES agents(agent_id),
    path TEXT, 
    PRIMARY KEY (agent_id)
);
