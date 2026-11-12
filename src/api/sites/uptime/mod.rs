mod site_figures;
mod timeline;
mod window;

pub(crate) use site_figures::set_site_uptime_figures;
pub(super) use timeline::OutageTimes;
pub(super) use window::{UptimeWindow, daily_uptime, uptime_window_start, window_summary};

#[cfg(test)]
#[path = "../../../tests/api/sites/uptime.rs"]
mod tests;
