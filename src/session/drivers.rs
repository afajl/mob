use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drivers(Vec<String>);

impl Drivers {
    pub fn new(drivers: Vec<String>) -> Self {
        Drivers(drivers)
    }

    pub fn insert(mut self, after: Option<String>, name: &str) -> Self {
        if self.contains(name) {
            return self;
        }

        let index = match after {
            Some(after) => self.0.iter().position(|existing| existing == &after),
            None => None,
        };

        match index {
            Some(i) => {
                if i + 1 > self.0.len() {
                    self.0.push(name.to_string());
                } else {
                    self.0.insert(i + 1, name.to_string());
                }
            }
            None => self.0.push(name.to_string()),
        }
        self
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains(&name.to_string())
    }

    fn position(&self, driver: &str) -> Option<usize> {
        let len = self.0.len();

        if len == 0 {
            panic!("Trying to find driver before anyone started");
        }

        if len == 1 {
            return None;
        }

        Some(
            self.0
                .iter()
                .position(|name| name == driver)
                .expect("Could not find current driver in session"),
        )
    }

    pub fn next(&self, current: &str) -> Option<String> {
        self.position(current).map(|i| {
            let next_index = (i + 1) % self.0.len();
            self.0[next_index].clone()
        })
    }

    pub fn prev(&self, current: &str) -> Option<String> {
        self.position(current).map(|i| {
            let prev_index = if i == 0 { self.0.len() - 1 } else { i - 1 };
            self.0[prev_index].clone()
        })
    }

    pub fn remove(mut self, name: &str) -> Self {
        let index = self
            .position(name)
            .expect("Trying to remove driver that is not part of the session");
        self.0.remove(index);
        self
    }

    pub fn all(&self) -> Vec<String> {
        self.0.clone()
    }
}

impl Default for Drivers {
    fn default() -> Self {
        Drivers(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_at_end() {
        let drivers = Drivers::new(vec!["a".to_string(), "b".to_string()]);
        let driver_added = drivers.insert(Some("b".to_string()), "c");
        assert_eq!(
            driver_added.all(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn insert_in_middle() {
        let drivers = Drivers::new(vec!["a".to_string(), "b".to_string()]);
        let driver_added = drivers.insert(Some("a".to_string()), "c");
        assert_eq!(
            driver_added.all(),
            vec!["a".to_string(), "c".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn remove() {
        let drivers = Drivers::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let driver_removed = drivers.remove("c");
        assert_eq!(driver_removed.all(), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn remove_middle() {
        let drivers = Drivers::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let driver_removed = drivers.remove("b");
        assert_eq!(driver_removed.all(), vec!["a".to_string(), "c".to_string()]);
    }
}
