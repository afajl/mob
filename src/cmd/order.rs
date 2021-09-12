use crate::session;
use anyhow::Result;
use dialoguer::{Confirm, Select, Sort};

pub struct Order<'a> {
    store: &'a dyn session::Store,
}

impl<'a> Order<'a> {
    pub fn new(store: &'a impl session::Store) -> Order<'a> {
        Self { store }
    }

    pub fn run(&self) -> Result<()> {
        let session = self.store.load()?;

        let drivers = session.drivers.all();

        if let Some(msg) = match drivers.len() {
            0 => Some("No drivers to reorder, run `mob start` first"),
            1 => Some("You're alone"),
            _ => None,
        } {
            log::info!("{}", msg);
            return Ok(());
        }

        let order = Sort::new()
            .with_prompt("Use [space] and ↓↑ to move driver")
            .items(drivers.as_slice())
            .interact()?;

        let ordered_drivers =
            session::Drivers::new(order.into_iter().map(|i| drivers[i].clone()).collect());

        let state = match session.state {
            session::State::WaitingForNext {
                next: Some(old_next),
            } => {
                // If we already are waiting for the next driver we potentially need to change iter
                // to the next one in the new order
                let previous_driver = drivers
                    .iter()
                    .position(|name| name == &old_next)
                    .map(|index| {
                        let prev_index = if index == 0 {
                            drivers.len() - 1
                        } else {
                            index - 1
                        };
                        drivers[prev_index].clone()
                    })
                    .expect("Previous driver not found, this should not happen");

                let next_driver = ordered_drivers.next(previous_driver.as_str());
                let next_driver_name = next_driver.as_ref().unwrap();

                let next_driver = if Confirm::new()
                    .with_prompt(format!("So {} should be next?", next_driver_name))
                    .interact()?
                {
                    next_driver
                } else {
                    let ordered = ordered_drivers.all();
                    let next = Select::new()
                        .with_prompt("Who should be next?")
                        .items(ordered.as_slice())
                        .interact()?;

                    Some(ordered[next].clone())
                };

                log::info!("Next driver: {}", next_driver.as_ref().unwrap());

                session::State::WaitingForNext { next: next_driver }
            }
            _ => session.state,
        };

        let session = session::Session {
            drivers: ordered_drivers,
            state,
            ..session
        };

        self.store.save(session)?;

        Ok(())
    }
}
