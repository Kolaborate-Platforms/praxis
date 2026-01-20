//! Agent module - orchestration and conversation management
//!
//! Contains the main agent logic that coordinates LLM calls and tool execution.

pub mod conversation;
pub mod loop_state;
pub mod orchestrator;
pub mod sub_agent;

pub use conversation::Conversation;
pub use loop_state::{AgentLoopState, Observation};
pub use orchestrator::Agent;
pub use sub_agent::{SubAgent, SubAgentBuilder, SubAgentManager};
