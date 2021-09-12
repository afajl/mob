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
            Some(i) => self.0.insert((i + 1) % self.0.len(), name.to_string()),
            None => self.0.push(name.to_string()),
        }
        self
    }

    pub fn contains(&self, name: &str) -> bool {
        return self.0.contains(&name.to_string());
    }

    pub fn next(&self, current: &str) -> Option<String> {
        match self.0.len() {
            0 => panic!("Next driver called before anyone started"),
            1 => None,
            _ => Some(
                self.0
                    .iter()
                    .position(|name| name == current)
                    .map(|index| {
                        let next_index = (index + 1) % self.0.len();
                        self.0[next_index].clone()
                    })
                    .expect(
                        format!("Could not find current driver {} in drivers", current).as_str(),
                    ),
            ),
        }
    }

    pub fn remove(mut self, name: &str) -> Self {
        let index = self
            .0
            .iter()
            .position(|n| n == name)
            .expect(format!("Trying to remove {} that is not a driver", name).as_str());
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
