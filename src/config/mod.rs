pub mod settings;
pub mod state;

pub struct Config<'a> {
    pub break_duration: i64,
    pub lunch_start: &'a str,
}

impl<'a> Config<'a> {
    pub fn from(mob: &'a settings::Settings) -> Config<'a> {
        Config {
            break_duration: mob.break_duration,
            lunch_start: mob.lunch_start.as_str(),
        }
    }
}
