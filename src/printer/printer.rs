pub(crate) const NEW_LINE: &'static str = "\n";
pub(crate) const TEXT_BOLD_ON: &'static str = "\x1b[1m";
pub(crate) const TEXT_BOLD_OFF: &'static str = "\x1b[m";

pub trait Printer: ExecutionResultPrinter {
    fn bold_text(&self, s: &str) -> String {
        format!("{}{}{}", s, TEXT_BOLD_ON, TEXT_BOLD_OFF)
    }

    fn print(&self) {
        print!("{}", self.buffer());
    }
}

pub trait ExecutionResultPrinter {
    fn is_error(&self) -> bool;

    fn buffer(&self) -> String;
}
