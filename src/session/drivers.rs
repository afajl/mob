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

    pub fn next(&self, current: &str) -> Option<String> {
        match self.0.len() {
            0 => panic!("Next driver called before anyone started"),
            1 => None,
            _ => Some(self.0.iter().position(|name| name == current).map_or_else(
                || panic!("Could not find current driver {} in drivers", current),
                |index| {
                    let next_index = (index + 1) % self.0.len();
                    self.0[next_index].clone()
                },
            )),
        }
    }

    pub fn remove(mut self, name: &str) -> Self {
        let index = self
            .0
            .iter()
            .position(|n| n == name)
            .unwrap_or_else(|| panic!("Trying to remove {} that is not a driver", name));
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
}
