pub mod app;
pub mod timer;
pub mod romodoro;
pub mod settings;
pub mod ui;
pub mod stats;
pub mod error;
pub const DEFAULT_WORK: i64 = 1800;
pub const DEFAULT_ITERATIONS: u8 = 4;
pub const DEFAULT_BREAK: i64 = 300;
pub const WORK_TIME_INCR: i64 = 900;
pub const BREAK_TIME_INCR: i64 = 60;
