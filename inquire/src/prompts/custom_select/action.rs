use crate::{
    ui::{Key, KeyModifiers},
    InnerAction, InputAction,
};

use super::config::CustomSelectConfig;

/// Set of actions for a CustomSelectPrompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CustomSelectPromptAction {
    /// Action on the value text input handler.
    FilterInput(InputAction),
    /// Moves the cursor to the option above.
    MoveUp,
    /// Moves the cursor to the option below.
    MoveDown,
    /// Moves the cursor to the page above.
    PageUp,
    /// Moves the cursor to the page below.
    PageDown,
    /// Moves the cursor to the start of the list.
    MoveToStart,
    /// Moves the cursor to the end of the list.
    MoveToEnd,
}

impl InnerAction for CustomSelectPromptAction {
    type Config = CustomSelectConfig;

    fn from_key(key: Key, config: &CustomSelectConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(Self::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::MoveDown),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => Self::MoveUp,
            Key::PageUp => Self::PageUp,
            Key::Home => Self::MoveToStart,

            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) => Self::MoveDown,
            Key::PageDown => Self::PageDown,
            Key::End => Self::MoveToEnd,

            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::FilterInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
