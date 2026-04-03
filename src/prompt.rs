use anyhow::Result;
use std::cell::RefCell;

/// Trait for interactive prompts, abstracting dialoguer for testability
pub trait Prompter {
    /// Display a selection menu and return the chosen index
    fn select(&self, items: &[&str], default: usize) -> Result<usize>;

    /// Display a selection menu with a prompt message
    fn select_with_prompt(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize>;

    /// Prompt for string input with a default value
    fn input_string(&self, prompt: &str, default: &str) -> Result<String>;

    /// Prompt for numeric input with a default value
    fn input_i64(&self, prompt: &str, default: i64) -> Result<i64>;

    /// Prompt for confirmation (yes/no)
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool>;

    /// Display a sortable list and return the new order as indices
    fn sort(&self, prompt: &str, items: &[&str]) -> Result<Vec<usize>>;
}

/// Real implementation using dialoguer
pub struct DialoguerPrompter;

impl Prompter for DialoguerPrompter {
    fn select(&self, items: &[&str], default: usize) -> Result<usize> {
        Ok(dialoguer::Select::new()
            .default(default)
            .items(items)
            .interact()?)
    }

    fn select_with_prompt(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize> {
        Ok(dialoguer::Select::new()
            .with_prompt(prompt)
            .default(default)
            .items(items)
            .interact()?)
    }

    fn input_string(&self, prompt: &str, default: &str) -> Result<String> {
        Ok(dialoguer::Input::new()
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()?)
    }

    fn input_i64(&self, prompt: &str, default: i64) -> Result<i64> {
        Ok(dialoguer::Input::new()
            .with_prompt(prompt)
            .default(default)
            .interact_text()?)
    }

    fn confirm(&self, prompt: &str, default: bool) -> Result<bool> {
        Ok(dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact()?)
    }

    fn sort(&self, prompt: &str, items: &[&str]) -> Result<Vec<usize>> {
        Ok(dialoguer::Sort::new()
            .with_prompt(prompt)
            .items(items)
            .interact()?)
    }
}

/// Mock prompter that returns preset answers for testing
pub struct MockPrompter {
    selections: RefCell<Vec<usize>>,
    strings: RefCell<Vec<String>>,
    numbers: RefCell<Vec<i64>>,
    confirms: RefCell<Vec<bool>>,
    sorts: RefCell<Vec<Vec<usize>>>,
}

impl MockPrompter {
    pub fn new() -> Self {
        Self {
            selections: RefCell::new(Vec::new()),
            strings: RefCell::new(Vec::new()),
            numbers: RefCell::new(Vec::new()),
            confirms: RefCell::new(Vec::new()),
            sorts: RefCell::new(Vec::new()),
        }
    }

    /// Queue a selection response
    pub fn with_selection(self, index: usize) -> Self {
        self.selections.borrow_mut().push(index);
        self
    }

    /// Queue a string input response
    pub fn with_string(self, s: &str) -> Self {
        self.strings.borrow_mut().push(s.to_string());
        self
    }

    /// Queue a numeric input response
    pub fn with_number(self, n: i64) -> Self {
        self.numbers.borrow_mut().push(n);
        self
    }

    /// Queue a confirmation response
    pub fn with_confirm(self, b: bool) -> Self {
        self.confirms.borrow_mut().push(b);
        self
    }

    /// Queue a sort response (list of indices representing new order)
    pub fn with_sort(self, order: Vec<usize>) -> Self {
        self.sorts.borrow_mut().push(order);
        self
    }
}

impl Default for MockPrompter {
    fn default() -> Self {
        Self::new()
    }
}

impl Prompter for MockPrompter {
    fn select(&self, _items: &[&str], default: usize) -> Result<usize> {
        Ok(self.selections.borrow_mut().pop().unwrap_or(default))
    }

    fn select_with_prompt(&self, _prompt: &str, _items: &[&str], default: usize) -> Result<usize> {
        Ok(self.selections.borrow_mut().pop().unwrap_or(default))
    }

    fn input_string(&self, _prompt: &str, default: &str) -> Result<String> {
        Ok(self
            .strings
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| default.to_string()))
    }

    fn input_i64(&self, _prompt: &str, default: i64) -> Result<i64> {
        Ok(self.numbers.borrow_mut().pop().unwrap_or(default))
    }

    fn confirm(&self, _prompt: &str, default: bool) -> Result<bool> {
        Ok(self.confirms.borrow_mut().pop().unwrap_or(default))
    }

    fn sort(&self, _prompt: &str, items: &[&str]) -> Result<Vec<usize>> {
        // Default: return items in original order
        Ok(self
            .sorts
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| (0..items.len()).collect()))
    }
}
