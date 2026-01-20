//! Conversation history management
//!
//! Maintains chat history with configurable limits.

use std::collections::VecDeque;

use crate::core::Message;

/// Manages conversation history
#[derive(Debug, Clone)]
pub struct Conversation {
    /// Message history
    messages: VecDeque<Message>,
    /// Maximum history length
    max_length: usize,
    /// System prompt (always first)
    system_prompt: Option<String>,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(max_length: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_length,
            system_prompt: None,
        }
    }

    /// Set the system prompt
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    /// Add a user message
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }

    /// Add an assistant message
    pub fn add_assistant(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }

    /// Add a message and maintain size limit
    fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);

        // Remove oldest messages if over limit (but keep recent context)
        while self.messages.len() > self.max_length {
            self.messages.pop_front();
        }
    }

    /// Get all messages including system prompt
    pub fn get_messages(&self) -> Vec<Message> {
        let mut result = Vec::new();

        if let Some(ref prompt) = self.system_prompt {
            result.push(Message::system(prompt.clone()));
        }

        result.extend(self.messages.iter().cloned());
        result
    }

    /// Get messages without system prompt
    pub fn get_history(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Get a specific range of messages from history
    ///
    /// If start or end are out of bounds, they are clamped to valid range.
    pub fn get_range(&self, start: usize, end: usize) -> Vec<Message> {
        let len = self.messages.len();
        if len == 0 {
            return Vec::new();
        }

        let start = start.min(len - 1);
        let end = end.min(len).max(start);

        self.messages
            .iter()
            .skip(start)
            .take(end - start)
            .cloned()
            .collect()
    }

    /// Get the last N messages
    pub fn last_n(&self, n: usize) -> Vec<&Message> {
        self.messages.iter().rev().take(n).rev().collect()
    }

    /// Get the last user message
    pub fn last_user_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == "user")
    }

    /// Get the last assistant message
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == "assistant")
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the context window (System prompt + last N messages)
    ///
    /// This effectively implements the RLM strategy: only the most recent context
    /// is loaded into the model's immediate window. The rest is available via
    /// the `analyze_conversation` tool.
    pub fn get_context_window(&self, window_size: usize) -> Vec<Message> {
        let mut result = Vec::new();

        if let Some(ref prompt) = self.system_prompt {
            result.push(Message::system(prompt.clone()));
        }

        let len = self.messages.len();
        let start = if len > window_size {
            len - window_size
        } else {
            0
        };

        result.extend(self.messages.iter().skip(start).cloned());

        result
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_basic() {
        let mut conv = Conversation::new(10);
        conv.add_user("Hello");
        conv.add_assistant("Hi there!");

        assert_eq!(conv.len(), 2);
        assert_eq!(conv.last_user_message().unwrap().content, "Hello");
    }

    #[test]
    fn test_conversation_limit() {
        let mut conv = Conversation::new(3);
        conv.add_user("1");
        conv.add_assistant("2");
        conv.add_user("3");
        conv.add_assistant("4");

        assert_eq!(conv.len(), 3);
        // First message should be removed
        assert_eq!(conv.messages[0].content, "2");
    }

    #[test]
    fn test_system_prompt() {
        let mut conv = Conversation::new(10);
        conv.set_system_prompt("You are a helpful assistant");
        conv.add_user("Hello");

        let messages = conv.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
    }
}
