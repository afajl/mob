use crate::{config::Config, duration, git, session, timer};
use anyhow::Result;
use chrono::{Duration, Local, NaiveTime, Utc};
use session::State;

enum Break<'a> {
    Yes(Duration, &'a str, session::Session),
    No(session::Session),
}

pub struct Next<'a> {
    git: &'a dyn git::Git,
    store: &'a dyn session::Store,
    timer: &'a dyn timer::Timer,
    config: Config,
}

impl<'a> Next<'a> {
    pub fn new(
        git: &'a impl git::Git,
        store: &'a impl session::Store,
        timer: &'a impl timer::Timer,
        config: Config,
    ) -> Next<'a> {
        Self {
            git,
            store,
            timer,
            config,
        }
    }

    pub fn run(&self) -> Result<()> {
        let me = &self.config.name;

        let session = self.store.load()?;
        match &session.state {
            State::Stopped => {
                log::warn!("No current mob session, run mob start");
            }
            State::Working { driver } if driver != me.as_str() => {
                log::warn!("The current driver is {}", driver);
            }
            State::Working { .. } => self.next(session)?,
            State::WaitingForNext { next, .. } => {
                match next {
                    Some(name) if name == me.as_str() => log::info!("It's your turn. Run start"),
                    Some(name) => log::info!("Waiting for {} to start", name),
                    None => log::info!("Waiting for someone to run start"),
                };
            }
        };
        Ok(())
    }

    fn next(&self, session: session::Session) -> Result<()> {
        if self.git.tree_is_clean()? {
            log::info!("Nothing was changed, so nothing to commit");
        } else {
            self.git.run(&["add", "--all"])?;
            self.git.run(&[
                "commit",
                "--message",
                session.settings.as_ref().unwrap().commit_message.as_str(),
                "--no-verify",
            ])?;

            self.git.run(&[
                "push",
                "--no-verify",
                self.config.remote.as_str(),
                session.branches.branch.as_str(),
            ])?;
        }

        let next_driver = session.drivers.next(&self.config.name);
        let next_driver_name = match next_driver {
            Some(ref driver) => driver,
            None => "anyone!",
        };

        let take_break = self.take_break_or_lunch(session)?;

        match take_break {
            Break::Yes(duration, name, session) => {
                let session = session::Session {
                    state: State::WaitingForNext {
                        next: next_driver.clone(),
                        is_break: true,
                    },
                    ..session
                };
                self.store.save(&session)?;

                let title = format!("{}, next driver {}", name, next_driver_name);
                let message = format!("{} over. {} turn", name, next_driver_name);

                self.timer
                    .start(title.as_str(), duration, message.as_str())?;
            }
            Break::No(session) => {
                let session = session::Session {
                    state: State::WaitingForNext {
                        next: next_driver.clone(),
                        is_break: false,
                    },
                    ..session
                };
                self.store.save(&session)?;
            }
        };

        log::info!("Next driver: {}", next_driver_name);
        Ok(())
    }

    fn take_break_or_lunch(&self, session: session::Session) -> Result<Break> {
        let take_lunch = self.take_lunch(session)?;
        Ok(match take_lunch {
            Break::Yes(..) => take_lunch,
            Break::No(session) => self.take_break(session)?,
        })
    }

    fn take_lunch(&self, session: session::Session) -> Result<Break> {
        let settings = session
            .settings
            .as_ref()
            .expect("Should have settings at this point");

        log::trace!(
            "Lunch parameters: now={}, lunch_start={}, work_duration={}",
            Local::now().time(),
            &settings.lunch_start,
            &settings.work_duration
        );

        let should_lunch = is_lunch_time(
            Local::now().time(),
            settings.work_duration,
            settings.lunch_start.as_str(),
            settings.lunch_end.as_str(),
        )?;

        if let Some(duration) = should_lunch {
            let take_lunch = dialoguer::Confirm::new()
                .with_prompt("It's lunch time. Go for lunch?")
                .default(true)
                .interact()?;
            if take_lunch {
                return Ok(Break::Yes(duration, "Lunch", session));
            }
        }

        Ok(Break::No(session))
    }

    fn take_break(&self, session: session::Session) -> Result<Break> {
        let settings = session
            .settings
            .as_ref()
            .expect("Should have settings at this point");

        log::trace!(
            "Break parameters: now={}, last_break={}, break_interval={}, work_duration={}",
            Local::now().time(),
            session.last_break,
            settings.break_interval,
            settings.work_duration
        );

        let should_break = is_break_time(
            Utc::now(),
            session.last_break,
            settings.break_interval,
            settings.break_duration,
            settings.work_duration,
        );

        if let Some(duration) = should_break {
            let last_break = duration::format(Utc::now() - session.last_break);

            let prompt = format!("It has been {} since the last break", last_break.human());

            let selections = &[
                "Take a break",
                "Just one more iteration...",
                "Skip this break",
            ];
            let selection = dialoguer::Select::new()
                .with_prompt(prompt)
                .default(0)
                .items(&selections[..])
                .interact()?;

            return match selection {
                0 => Ok(Break::Yes(duration, "Break", session)),
                1 => Ok(Break::No(session)),
                _ => Ok(Break::No(session::Session {
                    last_break: Utc::now(),
                    ..session
                })),
            };
        }

        Ok(Break::No(session))
    }
}

fn is_break_time(
    now: chrono::DateTime<Utc>,
    last_break: chrono::DateTime<Utc>,
    break_interval: i64,
    break_duration: i64,
    work_duration: i64,
) -> Option<Duration> {
    let duration_since_last = now - last_break;
    if duration_since_last
        > Duration::minutes(break_interval) + Duration::minutes(work_duration / 2)
    {
        return Some(Duration::minutes(break_duration));
    }
    None
}

fn is_lunch_time(
    now: chrono::NaiveTime,
    work_duration: i64,
    lunch_start: &str,
    lunch_end: &str,
) -> Result<Option<Duration>> {
    let lunch_start = NaiveTime::parse_from_str(lunch_start, "%H:%M")?;
    let lunch_end = NaiveTime::parse_from_str(lunch_end, "%H:%M")?;
    let work_duration = Duration::minutes(work_duration);
    let lunch_duration = lunch_end - lunch_start;

    let start_nagging = lunch_start - work_duration / 2;
    let end_nagging = lunch_start + work_duration;

    if now >= start_nagging && now < end_nagging {
        return Ok(Some(lunch_duration));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn break_before_work_duration() -> Result<()> {
        let now = DateTime::parse_from_rfc3339("1996-12-19T12:00:00-00:00")?.with_timezone(&Utc);
        let last_break =
            DateTime::parse_from_rfc3339("1996-12-19T11:00:00-00:00")?.with_timezone(&Utc);
        let break_interval = 55;
        let break_duration = 10;
        let work_duration = 9;
        let is_break = is_break_time(
            now,
            last_break,
            break_interval,
            break_duration,
            work_duration,
        );
        match is_break {
            Some(duration) => assert_eq!(duration.num_minutes(), break_duration),
            None => panic!("should break"),
        }
        Ok(())
    }

    #[test]
    fn break_after_work_duration() -> Result<()> {
        let now = DateTime::parse_from_rfc3339("1996-12-19T12:00:00-00:00")?.with_timezone(&Utc);
        let last_break =
            DateTime::parse_from_rfc3339("1996-12-19T11:00:00-00:00")?.with_timezone(&Utc);
        let break_interval = 55;
        let work_duration = 15;
        let break_duration = 10;
        let is_break = is_break_time(
            now,
            last_break,
            break_interval,
            break_duration,
            work_duration,
        );

        match is_break {
            Some(_) => panic!("should not break"),
            None => (),
        }
        Ok(())
    }

    #[test]
    fn break_for_lunch() -> Result<()> {
        let now = NaiveTime::parse_from_str("11:30", "%H:%M")?;
        let lunch_start = "11:30";
        let lunch_end = "12:30";
        let work_duration = 10;
        let is_lunch = is_lunch_time(now, work_duration, lunch_start.into(), lunch_end.into());
        match is_lunch {
            Ok(Some(duration)) => assert_eq!(duration.num_minutes(), 60),
            Ok(None) => panic!("Should lunch"),
            Err(err) => {
                dbg!(&err);
                panic!(err)
            }
        }
        Ok(())
    }

    #[test]
    fn before_lunch() -> Result<()> {
        let now = NaiveTime::parse_from_str("11:30", "%H:%M")?;
        let lunch_start = "11:40";
        let lunch_end = "12:30";
        let work_duration = 10;
        let is_lunch = is_lunch_time(now, work_duration, lunch_start.into(), lunch_end.into());
        match is_lunch {
            Ok(Some(_)) => panic!("should not lunch"),
            Ok(None) => (),
            Err(err) => {
                dbg!(&err);
                panic!(err)
            }
        }
        Ok(())
    }

    #[test]
    fn after_lunch() -> Result<()> {
        let now = NaiveTime::parse_from_str("12:10", "%H:%M")?;
        let lunch_start = "11:40";
        let lunch_end = "12:30";
        let work_duration = 10;
        let is_lunch = is_lunch_time(now, work_duration, lunch_start.into(), lunch_end.into());
        match is_lunch {
            Ok(Some(_)) => panic!("should not lunch"),
            Ok(None) => (),
            Err(err) => {
                dbg!(&err);
                panic!(err)
            }
        }
        Ok(())
    }
}
