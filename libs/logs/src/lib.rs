#[cfg(test)]
mod tests;

mod types;

pub use crate::types::LogFilter;
use ic_canister_log::{export as export_logs, GlobalBuffer, Sink};
use serde::Deserialize;
use std::str::FromStr;

pub trait LogPriority {
    fn get_buffer(&self) -> &'static GlobalBuffer;
    fn as_str_uppercase(&self) -> &'static str;
    fn get_priorities() -> &'static [Self]
    where
        Self: Sized;
    fn get_log_filter() -> LogFilter;
}

#[derive(Debug)]
pub struct PrintProxySink<Priority: 'static>(pub &'static Priority, pub &'static GlobalBuffer);

impl<Priority: LogPriority> Sink for PrintProxySink<Priority> {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        let message = format!(
            "{} {}:{} {}",
            self.0.as_str_uppercase(),
            entry.file,
            entry.line,
            entry.message,
        );
        if Priority::get_log_filter().is_match(&message) {
            ic_cdk::println!("{}", message);
            self.1.append(entry)
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, serde::Serialize)]
pub enum Sort {
    Ascending,
    Descending,
}

impl FromStr for Sort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(Sort::Ascending),
            "desc" => Ok(Sort::Descending),
            _ => Err("could not recognize sort order".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, serde::Serialize)]
pub struct LogEntry<Priority> {
    pub timestamp: u64,
    pub priority: Priority,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub counter: u64,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct Log<Priority> {
    pub entries: Vec<LogEntry<Priority>>,
}

impl<Priority> Default for Log<Priority> {
    fn default() -> Self {
        Self { entries: vec![] }
    }
}

impl<'de, Priority> Log<Priority>
where
    Priority: LogPriority + Clone + Copy + Deserialize<'de> + serde::Serialize + 'static,
{
    pub fn push_logs(&mut self, priority: Priority) {
        let logs = export_logs(priority.get_buffer());
        for entry in logs {
            self.entries.push(LogEntry {
                timestamp: entry.timestamp,
                counter: entry.counter,
                priority: priority.clone(),
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
    }

    pub fn push_all(&mut self) {
        Priority::get_priorities()
            .iter()
            .for_each(|priority| self.push_logs(*priority));
    }

    pub fn serialize_logs(&self, max_body_size: usize) -> String {
        let mut entries_json: String = serde_json::to_string(&self).unwrap_or_default();

        if entries_json.len() > max_body_size {
            let mut left = 0;
            let mut right = self.entries.len();

            while left < right {
                let mid = left + (right - left) / 2;
                let mut temp_log = self.clone();
                temp_log.entries.truncate(mid);
                let temp_entries_json = serde_json::to_string(&temp_log).unwrap_or_default();

                if temp_entries_json.len() <= max_body_size {
                    entries_json = temp_entries_json;
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
        }
        entries_json
    }

    pub fn sort_logs(&mut self, sort_order: Sort) {
        match sort_order {
            Sort::Ascending => self.sort_asc(),
            Sort::Descending => self.sort_desc(),
        }
    }

    pub fn sort_asc(&mut self) {
        self.entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    pub fn sort_desc(&mut self) {
        self.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
}
