//! Snapshot parsing for agent-browser output
//!
//! Parses the accessibility tree JSON from agent-browser.

use serde::{Deserialize, Serialize};

/// Parsed snapshot from agent-browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Whether the operation succeeded
    #[serde(default)]
    pub success: bool,
    /// Snapshot data
    #[serde(default)]
    pub data: Option<SnapshotData>,
}

/// Snapshot data content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotData {
    /// Raw snapshot string (accessibility tree)
    #[serde(default)]
    pub snapshot: String,
    /// Element refs mapped to their info
    #[serde(default)]
    pub refs: std::collections::HashMap<String, Element>,
}

/// An element in the snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// ARIA role
    #[serde(default)]
    pub role: String,
    /// Accessible name
    #[serde(default)]
    pub name: String,
    /// Element value (for inputs)
    #[serde(default)]
    pub value: Option<String>,
    /// Whether element is focused
    #[serde(default)]
    pub focused: bool,
    /// Additional properties
    #[serde(flatten)]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

impl Snapshot {
    /// Count the number of elements with refs
    pub fn count_elements(&self) -> usize {
        self.data.as_ref().map(|d| d.refs.len()).unwrap_or(0)
    }

    /// Get an element by ref
    pub fn get_element(&self, ref_id: &str) -> Option<&Element> {
        // Remove @ prefix if present
        let clean_ref = ref_id.strip_prefix('@').unwrap_or(ref_id);
        self.data.as_ref().and_then(|d| d.refs.get(clean_ref))
    }

    /// Get all interactive elements
    pub fn interactive_elements(&self) -> Vec<(&String, &Element)> {
        self.data
            .as_ref()
            .map(|d| {
                d.refs
                    .iter()
                    .filter(|(_, el)| {
                        matches!(
                            el.role.as_str(),
                            "button"
                                | "link"
                                | "textbox"
                                | "checkbox"
                                | "radio"
                                | "combobox"
                                | "menuitem"
                                | "tab"
                                | "switch"
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get elements by role
    pub fn elements_by_role(&self, role: &str) -> Vec<(&String, &Element)> {
        self.data
            .as_ref()
            .map(|d| d.refs.iter().filter(|(_, el)| el.role == role).collect())
            .unwrap_or_default()
    }

    /// Find elements containing text in their name
    pub fn find_by_text(&self, text: &str) -> Vec<(&String, &Element)> {
        let text_lower = text.to_lowercase();
        self.data
            .as_ref()
            .map(|d| {
                d.refs
                    .iter()
                    .filter(|(_, el)| el.name.to_lowercase().contains(&text_lower))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the raw accessibility tree string
    pub fn raw_tree(&self) -> Option<&str> {
        self.data.as_ref().map(|d| d.snapshot.as_str())
    }

    /// Format snapshot for display
    pub fn format_for_display(&self) -> String {
        if let Some(data) = &self.data {
            let mut output = String::new();
            output.push_str("Page Elements:\n");

            for (ref_id, element) in &data.refs {
                let value_str = element
                    .value
                    .as_ref()
                    .map(|v| format!(" = \"{}\"", v))
                    .unwrap_or_default();

                output.push_str(&format!(
                    "  @{}: {} \"{}\"{}",
                    ref_id, element.role, element.name, value_str
                ));

                if element.focused {
                    output.push_str(" [focused]");
                }

                output.push('\n');
            }

            output
        } else {
            "No snapshot data available".to_string()
        }
    }
}

impl Element {
    /// Check if this is an interactive element
    pub fn is_interactive(&self) -> bool {
        matches!(
            self.role.as_str(),
            "button"
                | "link"
                | "textbox"
                | "checkbox"
                | "radio"
                | "combobox"
                | "menuitem"
                | "tab"
                | "switch"
                | "searchbox"
        )
    }

    /// Check if this is an input element
    pub fn is_input(&self) -> bool {
        matches!(
            self.role.as_str(),
            "textbox" | "searchbox" | "combobox" | "spinbutton"
        )
    }

    /// Check if this is clickable
    pub fn is_clickable(&self) -> bool {
        matches!(
            self.role.as_str(),
            "button" | "link" | "menuitem" | "tab" | "checkbox" | "radio" | "switch"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_is_interactive() {
        let button = Element {
            role: "button".to_string(),
            name: "Submit".to_string(),
            value: None,
            focused: false,
            properties: Default::default(),
        };
        assert!(button.is_interactive());
        assert!(button.is_clickable());
        assert!(!button.is_input());
    }

    #[test]
    fn test_snapshot_get_element() {
        let mut refs = std::collections::HashMap::new();
        refs.insert(
            "e1".to_string(),
            Element {
                role: "button".to_string(),
                name: "Click me".to_string(),
                value: None,
                focused: false,
                properties: Default::default(),
            },
        );

        let snapshot = Snapshot {
            success: true,
            data: Some(SnapshotData {
                snapshot: String::new(),
                refs,
            }),
        };

        assert!(snapshot.get_element("e1").is_some());
        assert!(snapshot.get_element("@e1").is_some());
        assert!(snapshot.get_element("e2").is_none());
    }
}
