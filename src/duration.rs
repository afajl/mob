pub struct FormattedDuration(chrono::Duration);

impl FormattedDuration {
    pub fn clock(&self) -> String {
        let (h, m, s) = self.hms();
        if h > 0 {
            format!("{:>2}:{:0>2}", h, m)
        } else if m > 0 {
            format!("{:>2}:{:0>2}", m, s)
        } else {
            format!("{:>2}", s)
        }
    }

    pub fn human(&self) -> String {
        let (h, m, _) = self.hms();
        if h > 0 {
            format!("{} hours and {} minutes", h, m)
        } else if m > 0 {
            format!("{} minutes", m)
        } else {
            format!("less than a minute")
        }
    }

    fn hms(&self) -> (i64, i64, i64) {
        let h = self.0.num_hours();
        let m = self.0.num_minutes() - h * 60;
        let s = self.0.num_seconds() - m * 60;
        (h, m, s)
    }
}

pub fn format(duration: chrono::Duration) -> FormattedDuration {
    FormattedDuration(duration)
}
