use crate::CustomSelect;

/// Configuration settings used in the execution of a CustomSelectPrompt.
#[derive(Copy, Clone, Debug)]
pub struct CustomSelectConfig {
    /// Whether to use vim-style keybindings.
    pub vim_mode: bool,
    /// Page size of the list of options.
    pub page_size: usize,
}

impl<T> From<&CustomSelect<'_, T>> for CustomSelectConfig {
    fn from(value: &CustomSelect<'_, T>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            page_size: value.page_size,
        }
    }
}
