#[derive(Debug, Clone, Copy)]
pub enum Tabs {
    TimerTab,
    StatsTab,
}
impl Tabs {
    pub fn next(&mut self) {
        *self = match self {
            Tabs::TimerTab => Tabs::StatsTab,
            Tabs::StatsTab => Tabs::TimerTab,
        };
    }
    pub fn prev(&mut self) {
        *self = match self {
            Tabs::TimerTab => Tabs::StatsTab,
            Tabs::StatsTab => Tabs::TimerTab,
        };
    }
}
impl From<Tabs> for usize {
    fn from(value: Tabs) -> Self {
        match value {
            Tabs::TimerTab => 0,
            Tabs::StatsTab => 1,
        }
    }
}
impl From<&Tabs> for usize {
    fn from(value: &Tabs) -> Self {
        value.clone().into()
    }
}
