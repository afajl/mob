use serde::{Deserialize, Serialize};

// #[derive(Debug, PartialEq)]
// pub struct Stopped {}

// #[derive(Debug, PartialEq)]
// pub struct Working {
//     driver: String,
// }

// #[derive(Debug, PartialEq)]
// pub struct Break {
//     until: i64,
//     next: String,
// }

// #[derive(Debug, PartialEq)]
// pub struct WaitingForNext {
//     next: String,
// }

// #[derive(Debug, Serialize, Deserialize, PartialEq)]
// pub struct Branches {
//     branch: String,
//     base_branch: String,
// }

// #[derive(Debug, PartialEq)]
// pub struct State<S: PartialEq> {
//     state: S,
// }

// impl State<Stopped> {
//     pub fn new() -> State<Stopped> {
//         State { state: Stopped {} }
//     }

//     pub fn start(driver: &str) -> State<Working> {
//         State {
//             state: Working {
//                 driver: driver.to_string(),
//             },
//         }
//     }
// }

// impl State<Working> {
//     pub fn waiting(self, next: String) -> State<WaitingForNext> {
//         State {
//             state: WaitingForNext { next },
//         }
//     }

//     pub fn take_break(self, until: i64, next: &str) -> State<Break> {
//         State {
//             state: Break {
//                 until,
//                 next: next.to_string(),
//             },
//         }
//     }
// }

// #[derive(Debug, PartialEq)]
// pub enum States {
//     Stopped(State<Stopped>),
//     Working(State<Working>),
//     Break(State<Break>),
//     WaitingForNext(State<WaitingForNext>),
// }

// pub struct Session {
//     branches: Branches,
//     state: States,
// }

// impl Session {
//     fn new(branches: Branches) -> Self {
//         Session {
//             branches,
//             state: States::Stopped(State::new()),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_initial() {
//         let branches = Branches {
//             base_branch: "master".to_string(),
//             branch: "x".to_string(),
//         };
//         let stopped = Session::new(branches);

//         match stopped.state {
//             States::Stopped(_) => {}
//             _ => panic!("bad state"),
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Branches {
    branch: String,
    base_branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Stopped,
    Working { driver: String },
    Break { until: i64, next: String },
    WaitingForNext { next: String },
    Shitty(String),
}

// impl State {
//     pub fn start(self, driver: &str) -> State {
//         match self {
//             State::Stopped | State::Break{..} | State::WaitingForNext {..} => State::Working { driver.to_string() },
//             bad => State::Bad { reason: format!
//     }

// }

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    StartInitial { branches: Branches, driver: String },
    Start { driver: String },
    Next { next: String },
    TakeBreak { until: i64, next: String },
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    branches: Branches,
    state: State,
}

impl Session {
    pub fn next(self, event: Event) -> Self {
        match (self.state, event) {
            (State::Stopped, Event::StartInitial { branches, driver }) => Session {
                state: State::Working { driver },
                branches,
            },
            (State::Working { .. }, Event::Next { next }) => Session {
                state: State::WaitingForNext { next },
                ..self
            },

            (State::Working { .. }, Event::Done) => Session {
                state: State::Stopped,
                ..self
            },

            (State::Working { .. }, Event::TakeBreak { until, next }) => Session {
                state: State::Break { until, next },
                ..self
            },

            (State::Break { .. }, Event::Start { driver }) => Session {
                state: State::Working { driver },
                ..self
            },
            (State::Break { .. }, Event::Done) => Session {
                state: State::Stopped,
                ..self
            },
            (State::WaitingForNext { .. }, Event::Start { driver }) => Session {
                state: State::Working { driver },
                ..self
            },
            (State::WaitingForNext { .. }, Event::Done) => Session {
                state: State::Stopped,
                ..self
            },
            (s, e) => Session {
                state: State::Shitty(format!("Event {:?} unexpted in state {:?}", e, s)),
                ..self
            },
        }
    }
}
