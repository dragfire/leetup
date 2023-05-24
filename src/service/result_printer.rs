use colci::Color;
use log::info;

use crate::{model::SubmissionResult, Either};

pub trait Printer {
    /// Prints text
    fn print(&self);
}

#[derive(Debug)]
pub struct TestCaseResult {
    answer: String,
    expected_answer: String,
    is_correct: bool,
}

impl TestCaseResult {
    fn new(answer: String, expected_answer: String, is_correct: bool) -> Self {
        Self {
            answer,
            expected_answer,
            is_correct,
        }
    }
}

impl Printer for TestCaseResult {
    fn print(&self) {
        let text = format!(
            "Answer: {}\nExpected Answer: {}",
            self.answer, self.expected_answer
        );
        let colored_text = if self.is_correct {
            Color::Green(&text).make()
        } else {
            Color::Red(&text).make()
        };
        println!("{}", colored_text);
    }
}

pub struct TestCaseResults {
    submission_result: SubmissionResult,
    results: Vec<TestCaseResult>,
}

impl TestCaseResults {
    fn new(submission_result: SubmissionResult, results: Vec<TestCaseResult>) -> Self {
        Self {
            submission_result,
            results,
        }
    }

    fn get_answers(left: Option<&Either>, right: Option<&Either>) -> Vec<(String, String)> {
        match (left, right) {
            (Some(Either::Sequence(vec1)), Some(Either::Sequence(vec2))) => {
                let mut vec = vec2.clone();
                if vec2.is_empty() {
                    vec = std::iter::repeat("".to_string())
                        .take(vec1.len())
                        .collect::<Vec<_>>();
                }
                vec1.iter().cloned().zip(vec.iter().cloned()).collect()
            }
            _ => vec![],
        }
    }

    fn print_compile_error(&self) {
        println!(
            "{}",
            Color::Red(
                &self
                    .submission_result
                    .full_compile_error
                    .to_owned()
                    .unwrap_or_default()
            )
            .make()
        );
    }

    fn print_status_msg(&self) {
        println!(
            "{}",
            Color::Red(self.submission_result.status_msg.as_str()).make()
        );
    }

    fn print_submission_result(&self) {
        let memory_percentile = self
            .submission_result
            .memory_percentile
            .unwrap_or(0.0)
            .to_string();
        let runtime_percentile = self
            .submission_result
            .runtime_percentile
            .unwrap_or(0.0)
            .to_string();
        let testcases = format!(
            "{}/{}",
            self.submission_result.total_correct.unwrap_or(0),
            self.submission_result.total_testcases.unwrap_or(0)
        );
        let accepted_meta = format!(
            "{}: ({})",
            self.submission_result.status_msg.as_str(),
            testcases.as_str()
        );
        let metas = vec![
            accepted_meta,
            "Memory: ".to_string() + self.submission_result.status_memory.as_str(),
            "Memory %ile: ".to_string() + memory_percentile.as_str(),
            "Runtime: ".to_string() + self.submission_result.status_runtime.as_str(),
            "Runtime %ile: ".to_string() + runtime_percentile.as_str(),
        ];
        for meta in metas {
            println!("{}", Color::Green(meta.as_str()).make());
        }
    }

    fn get_stdout(&self) -> String {
        if let Some(Either::Sequence(o)) = self.submission_result.code_output.as_ref() {
            return o.join("\n");
        }

        "".to_string()
    }
}

impl Printer for TestCaseResults {
    fn print(&self) {
        if self.submission_result.has_compile_error() {
            self.print_compile_error();
            return;
        }

        if !self.results.is_empty() {
            for (i, test_case_result) in self.results.iter().enumerate() {
                println!("\nTest {}/{}", i + 1, self.results.len());
                test_case_result.print();
            }

            println!(
                "\n{}\n{}",
                Color::Yellow("Output:").make(),
                Color::Magenta(&self.get_stdout()).make()
            );
            return;
        }

        if self.submission_result.has_runtime_error() || self.submission_result.has_error() {
            self.print_status_msg();
            return;
        }

        self.print_submission_result();
    }
}

impl From<SubmissionResult> for TestCaseResults {
    fn from(submission_result: SubmissionResult) -> Self {
        info!("submission result: {:#?}", submission_result);

        let answers = TestCaseResults::get_answers(
            submission_result.code_answer.as_ref(),
            submission_result.expected_code_answer.as_ref(),
        );

        let results = answers
            .iter()
            .map(|test_case_result| {
                let (answer, expected_answer) = test_case_result;
                TestCaseResult::new(
                    answer.to_string(),
                    expected_answer.to_string(),
                    answer.eq(expected_answer),
                )
            })
            .collect();

        TestCaseResults::new(submission_result, results)
    }
}
