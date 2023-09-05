use colci::Color;

use crate::printer::{ExecutionResultPrinter, Printer, NEW_LINE};
use crate::{icon::Icon, model::SubmissionResult, Either};

#[derive(Debug)]
pub struct TestExecutionResult {
    test_data: Either,
    submission_result: SubmissionResult,
}

impl Printer for TestExecutionResult {}

impl TestExecutionResult {
    pub fn new(test_data: Either, submission_result: SubmissionResult) -> Self {
        Self {
            test_data,
            submission_result,
        }
    }

    fn error_buffer(&self) -> String {
        let error_buffer =
            self.get_runtime_error() + NEW_LINE + NEW_LINE + self.get_compile_error().as_str();
        if error_buffer.trim().is_empty() {
            self.get_wrong_answer()
        } else {
            error_buffer
        }
    }

    fn get_runtime_error(&self) -> String {
        if !self.submission_result.has_runtime_error() {
            return NEW_LINE.to_owned();
        }
        self.submission_result.status_msg.to_owned()
    }

    fn get_compile_error(&self) -> String {
        if !self.submission_result.has_compile_error() {
            return NEW_LINE.to_owned();
        }
        self.submission_result
            .full_compile_error
            .to_owned()
            .unwrap_or_default()
    }

    fn get_wrong_answer(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&self.bold_text(
            &Color::Red(&format!("\n{} Wrong Answer:\n\n", Icon::_No.to_string())).make(),
        ));
        buffer.push_str(&self.test_cases_buffer());
        buffer.push_str(&Color::Red(&self.get_metas()).make());

        buffer
    }

    fn test_cases_buffer(&self) -> String {
        let mut buffer = String::new();
        // combine test_data, code_answer & expected_code_answer
        match (
            &self.test_data,
            &self.submission_result.code_answer,
            &self.submission_result.expected_code_answer,
        ) {
            (
                Either::Sequence(input_seq),
                Some(Either::Sequence(ans_seq)),
                Some(Either::Sequence(exp_ans_seq)),
            ) => {
                for (i, ((input, ans), exp_ans)) in
                    input_seq.iter().zip(ans_seq).zip(exp_ans_seq).enumerate()
                {
                    let mut test_case = String::new();
                    let is_correct = ans.eq(exp_ans);
                    let colored_case = if is_correct {
                        Color::Green(&format!("{} Case {}:\n", Icon::Yes.to_string(), i + 1)).make()
                    } else {
                        Color::Red(&format!("{} Case {}:\n", Icon::_No.to_string(), i + 1)).make()
                    };
                    test_case.push_str(&colored_case);
                    test_case.push_str(&format!("\tInput: {}\n", input));
                    test_case.push_str(&format!("\tOutput: {}\n", ans));
                    test_case.push_str(&format!("\tExpected: {}\n\n", exp_ans));

                    buffer.push_str(test_case.as_str());
                }
            }
            _ => {}
        }

        buffer
    }

    fn success(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(
            &self.bold_text(
                &Color::Green(&format!("{} Accepted:\n\n", Icon::Yes.to_string())).make(),
            ),
        );
        buffer.push_str(&self.test_cases_buffer());
        buffer.push_str(&Color::Green(&self.get_metas()).make());

        buffer
    }

    fn get_metas(&self) -> String {
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
            "\n\n".to_string(),
        ];

        metas.join("\n")
    }
}

impl ExecutionResultPrinter for TestExecutionResult {
    fn is_error(&self) -> bool {
        self.submission_result.has_compile_error()
            || self.submission_result.has_runtime_error()
            || self.submission_result.has_error()
    }

    fn buffer(&self) -> String {
        if self.is_error() {
            self.error_buffer()
        } else {
            self.success()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Printer, TestExecutionResult};
    use crate::{model::SubmissionResult, Either};
    use serde_json::from_value;

    #[test]
    fn print_sequence_success() {
        let test_data = Either::Sequence(vec![
            "[1,2,3]".to_owned(),
            "[1,0,-1,-1,-1,1,0,1]".to_owned(),
            "[1,0,-1]".to_owned(),
            "[-1,0,1,2,-1,-4]".to_owned(),
            "[0,1,1]".to_owned(),
            "[0,0,0]".to_owned(),
        ]);
        let json_value = serde_json::from_str(
            r#"{
	"status_code": 10,
	"lang": "java",
	"run_success": true,
	"status_runtime": "3 ms",
	"memory": 40404000,
	"display_runtime": "3",
	"code_answer": [
		"[]",
		"[[-1,0,1]]",
		"[[-1,0,1]]",
		"[[-1,-1,2],[-1,0,1]]",
		"[]",
		"[[0,0,0]]"
	],
	"code_output": [],
	"std_output_list": [
		"",
		"",
		"",
		"",
		"",
		"",
		""
	],
	"elapsed_time": 217,
	"task_finish_time": 1693886584999,
	"task_name": "judger.runcodetask.RunCode",
	"expected_status_code": 10,
	"expected_lang": "cpp",
	"expected_run_success": true,
	"expected_status_runtime": "0",
	"expected_memory": 6304000,
	"expected_code_answer": [
		"[]",
		"[[-1,0,1]]",
		"[[-1,0,1]]",
		"[[-1,-1,2],[-1,0,1]]",
		"[]",
		"[[0,0,0]]"
	],
	"expected_code_output": [],
	"expected_std_output_list": [
		"",
		"",
		"",
		"",
		"",
		"",
		""
	],
	"expected_elapsed_time": 21,
	"expected_task_finish_time": 1693885714905,
	"expected_task_name": "judger.interprettask.Interpret",
	"correct_answer": true,
	"compare_result": "111111",
	"total_correct": 6,
	"total_testcases": 6,
	"runtime_percentile": null,
	"status_memory": "40.4 MB",
	"memory_percentile": null,
	"pretty_lang": "Java",
	"submission_id": "runcode_1693886582.3348386_GUkqbCdnmN",
	"status_msg": "Accepted",
	"state": "SUCCESS"
}"#,
        )
        .unwrap();

        let response = from_value::<SubmissionResult>(json_value).unwrap().into();

        let result = TestExecutionResult::new(test_data, response);
        result.print();
        assert!(1 == 1);
    }

    #[test]
    fn print_sequence_wrong_answer() {
        let test_data = Either::Sequence(vec![
            "[1,2,3]".to_owned(),
            "[1,0,-1,-1,-1,1,0,1]".to_owned(),
            "[1,0,-1]".to_owned(),
            "[-1,0,1,2,-1,-4]".to_owned(),
            "[0,1,1]".to_owned(),
            "[0,0,0]".to_owned(),
        ]);
        let json_value = serde_json::from_str(
            r#"{
	"status_code": 10,
	"lang": "java",
	"run_success": true,
	"status_runtime": "2 ms",
	"memory": 40560000,
	"display_runtime": "2",
	"code_answer": [
		"[]",
		"[[-1,0,1]]",
		"[[-1,0,1]]",
		"[[-1,-1,2]]",
		"[]",
		"[[0,0,0]]"
	],
	"code_output": [],
	"std_output_list": [
		"",
		"",
		"",
		"",
		"",
		"",
		""
	],
	"elapsed_time": 229,
	"task_finish_time": 1693885714946,
	"task_name": "judger.runcodetask.RunCode",
	"expected_status_code": 10,
	"expected_lang": "cpp",
	"expected_run_success": true,
	"expected_status_runtime": "0",
	"expected_memory": 6304000,
	"expected_code_answer": [
		"[]",
		"[[-1,0,1]]",
		"[[-1,0,1]]",
		"[[-1,-1,2],[-1,0,1]]",
		"[]",
		"[[0,0,0]]"
	],
	"expected_code_output": [],
	"expected_std_output_list": [
		"",
		"",
		"",
		"",
		"",
		"",
		""
	],
	"expected_elapsed_time": 21,
	"expected_task_finish_time": 1693885714905,
	"expected_task_name": "judger.interprettask.Interpret",
	"correct_answer": false,
	"compare_result": "111011",
	"total_correct": 5,
	"total_testcases": 6,
	"runtime_percentile": null,
	"status_memory": "40.6 MB",
	"memory_percentile": null,
	"pretty_lang": "Java",
	"submission_id": "runcode_1693885710.9158015_9uDhRiWjFV",
	"status_msg": "Accepted",
	"state": "SUCCESS"
}"#,
        )
        .unwrap();

        let response = from_value::<SubmissionResult>(json_value).unwrap().into();

        let result = TestExecutionResult::new(test_data, response);
        result.print();
        assert!(1 == 1);
    }
}
