CREATE TABLE items (
  name TEXT NOT NULL,
  item_id TEXT PRIMARY KEY NOT NULL,
  asset_id TEXT, 
  parent_id TEXT, 
  description TEXT, 
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  inventory_type INTEGER, 
  flags INTEGER, 
  item_type TEXT,
  folder_id INTEGER REFERENCES folders(id),

  owner_id TEXT,
  group_id TEXT,
  creator_id TEXT,
  base_mask INTEGER,
  everyone_mask INTEGER,
  group_mask INTEGER,
  next_owner_mask INTEGER,
  owner_mask INTEGER,
  is_owner_group BOOLEAN,
  last_owner_id TEXT,
  
  sale_type TEXT, 
  price INTEGER,
  ownership_cost INTEGER

);

CREATE TABLE folders (
  id TEXT PRIMARY KEY NOT NULL,
  owner_id TEXT,
  agent_id TEXT,
  version INTEGER DEFAULT 1, 
  descendent_count INTEGER,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  parent TEXT REFERENCES folders(id)
);

CREATE TABLE categories (
  id TEXT PRIMARY KEY NOT NULL, 
  name TEXT NOT NULL, 
  type_default TEXT, 
  version INTEGER DEFAULT 1, 
  folder_id TEXT REFERENCES folders(id) ON DELETE CASCADE
)
