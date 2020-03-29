pub mod mob;

pub struct Config<'a> {
    pub break_duration: i64,
    pub lunch_start: &'a str,
    pub mob_branch: &'a str,
    pub base_branch: &'a str,
    pub remote: &'a str,
    pub remote_mob_branch: String,
    pub remote_base_branch: String,
}

impl<'a> Config<'a> {
    pub fn from(mob: &'a mob::Config) -> Config<'a> {
        let remote_mob_branch = [mob.remote.clone(), mob.mob_branch.clone()].join("/");
        let remote_base_branch = [mob.remote.clone(), mob.base_branch.clone()].join("/");
        Config {
            break_duration: mob.break_duration,
            lunch_start: mob.lunch_start.as_str(),
            mob_branch: mob.mob_branch.as_str(),
            base_branch: mob.base_branch.as_str(),
            remote: mob.remote.as_str(),
            remote_base_branch: remote_base_branch,
            remote_mob_branch: remote_mob_branch,
        }
    }
}
