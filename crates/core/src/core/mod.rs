/// Handles mailbox events to do with handling agents
pub mod agent;
/// Handles mailbox events required for establishing viewer capabilities
pub mod capabilities;
/// Handles mailbox events for generating land and environment
pub mod environment;
/// Generates GLTF files out of Mesh objects
pub mod generate_gltf;
/// Handles mailbox events for handling and updating inventory
pub mod inventory;
/// Handles mailbox events for retrieving and rendering objects
pub mod objects;
/// Handles mailbox events required for opening and maintaining the session
pub mod session;
