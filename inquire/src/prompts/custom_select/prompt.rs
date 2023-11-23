use std::{cmp::Reverse, fmt::Display};

use crate::{
    error::InquireResult,
    formatter::OptionFormatter,
    input::{Input, InputActionResult},
    list_option::ListOption,
    prompts::prompt::{ActionResult, Prompt},
    type_aliases::Scorer,
    ui::SelectBackend,
    utils::{paginate, Page},
    CustomSelect, InquireError,
};

use super::{action::CustomSelectPromptAction, config::CustomSelectConfig};

struct Window {
    offset: usize,
    window_length: usize,
    total_length: usize,
    cursor_index: usize,
}

pub trait OptionFetcher<T> {
    fn fetch(&self, input: &str, offset: usize, amount: usize) -> (Vec<T>, usize);
}

pub struct CustomSelectPrompt<'a, T> {
    message: &'a str,
    config: CustomSelectConfig,
    options_fetcher: Box<dyn OptionFetcher<T>>,
    fetched_options: Vec<T>,
    window: Window,
    help_message: Option<&'a str>,
    input: Input,
    formatter: OptionFormatter<'a, T>,
}

impl<'a, T> CustomSelectPrompt<'a, T>
where
    T: Display + 'static,
{
    pub fn new(so: CustomSelect<'a, T>) -> InquireResult<Self> {
        Ok(Self {
            message: so.message,
            config: CustomSelectConfig {
                vim_mode: so.vim_mode,
                page_size: so.page_size,
            },
            fetched_options: vec![],
            window: Window {
                offset: so.starting_cursor,
                window_length: so.page_size,
                total_length: 0,
                cursor_index: so.starting_cursor,
            },
            options_fetcher: so.options_fetcher,
            help_message: so.help_message,
            input: so
                .starting_filter_input
                .map(Input::new_with)
                .unwrap_or_else(Input::new),
            formatter: so.formatter,
        })
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) -> ActionResult {
        if self.window.total_length == 0 {
            return ActionResult::Clean;
        }

        let qty = qty % self.window.total_length;
        let new_index =
            (self.window.cursor_index + self.window.total_length - qty) % self.window.total_length;
        self.update_cursor_position(new_index)
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) -> ActionResult {
        if self.window.total_length == 0 {
            return ActionResult::Clean;
        }

        let new_index = (self.window.cursor_index + qty) % self.window.total_length;
        self.update_cursor_position(new_index)
    }

    fn update_cursor_position(&mut self, new_position: usize) -> ActionResult {
        if new_position != self.window.cursor_index {
            self.window.cursor_index = new_position;
            self.window.offset = self
                .window
                .cursor_index
                .min(
                    self.window
                        .total_length
                        .saturating_sub(self.window.window_length),
                )
                .min(new_position.saturating_sub(self.window.window_length / 2));
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    fn has_answer_highlighted(&mut self) -> bool {
        self.fetched_options
            .get(self.fetched_index_from_cursor_index())
            .is_some()
    }

    fn fetched_index_from_cursor_index(&self) -> usize {
        self.window.cursor_index.saturating_sub(self.window.offset)
    }

    fn get_final_answer(&mut self) -> ListOption<T> {
        // should only be called after current cursor index is validated
        // on has_answer_highlighted

        let index = self.fetched_index_from_cursor_index();
        let value = self.fetched_options.swap_remove(index);

        ListOption::new(index, value)
    }

    fn refetch(&mut self) {
        let (options, total_length) = self.options_fetcher.fetch(
            self.input.content(),
            self.window.offset,
            self.window.window_length,
        );

        self.fetched_options = options;
        self.window.total_length = total_length;

        self.update_cursor_position(
            self.window
                .cursor_index
                .max(self.window.offset)
                .min(self.window.offset + self.window.window_length)
                .min(self.window.total_length.saturating_sub(1)),
        );
    }
}

impl<'a, Backend, T> Prompt<Backend> for CustomSelectPrompt<'a, T>
where
    Backend: SelectBackend,
    T: Display + 'static,
{
    type Config = CustomSelectConfig;
    type InnerAction = CustomSelectPromptAction;
    type Output = ListOption<T>;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &CustomSelectConfig {
        &self.config
    }

    fn format_answer(&self, answer: &ListOption<T>) -> String {
        (self.formatter)(answer.as_ref())
    }

    fn setup(&mut self) -> InquireResult<()> {
        self.refetch();
        Ok(())
    }

    fn submit(&mut self) -> InquireResult<Option<ListOption<T>>> {
        let answer = match self.has_answer_highlighted() {
            true => Some(self.get_final_answer()),
            false => None,
        };

        Ok(answer)
    }

    fn handle(&mut self, action: CustomSelectPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            CustomSelectPromptAction::MoveUp => {
                let result = self.move_cursor_up(1, true);
                self.refetch();
                result
            }
            CustomSelectPromptAction::MoveDown => {
                let result = self.move_cursor_down(1, true);
                self.refetch();
                result
            }
            CustomSelectPromptAction::PageUp => todo!(),
            CustomSelectPromptAction::PageDown => todo!(),
            CustomSelectPromptAction::MoveToStart => todo!(),
            CustomSelectPromptAction::MoveToEnd => todo!(),
            CustomSelectPromptAction::FilterInput(input_action) => {
                let result = self.input.handle(input_action);

                if let InputActionResult::ContentChanged = result {
                    self.refetch();
                }

                result.into()
            }
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        backend.render_select_prompt(prompt, &self.input)?;

        let options: Vec<_> = self
            .fetched_options
            .iter()
            .enumerate()
            .map(|(i, el)| ListOption::new(self.window.offset + i, el))
            .collect();
        let page = Page {
            first: self.window.offset == 0,
            last: self.window.offset + self.window.window_length >= self.window.total_length,
            content: &options,
            cursor: Some(self.fetched_index_from_cursor_index()),
            total: self.window.total_length,
        };

        backend.render_options(page)?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        Ok(())
    }
}
