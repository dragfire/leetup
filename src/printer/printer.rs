use crate::model::SubmissionResponse;

pub(crate) const NEW_LINE: &'static str = "\n";
pub(crate) const TEXT_BOLD_ON: &'static str = "\x1b[1m";
pub(crate) const TEXT_BOLD_OFF: &'static str = "\x1b[m";

pub trait Printer {
    fn print(&self) {
        print!("{}", self.buffer());
    }

    fn is_error(&self) -> bool;

    fn buffer(&self) -> String;

    fn total_cases_ratio_buffer(&self, response: &SubmissionResponse) -> String {
        format!(
            "{}/{}",
            response.total_correct.unwrap_or(0),
            response.total_testcases.unwrap_or(0)
        )
    }
}

pub mod decorator {
    use super::*;

    pub fn bold_text(s: &str) -> String {
        format!("{}{}{}", s, TEXT_BOLD_ON, TEXT_BOLD_OFF)
    }
}
