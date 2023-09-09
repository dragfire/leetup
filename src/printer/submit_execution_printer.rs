use colci::Color;

use crate::model::ExecutionErrorResponse;
use crate::printer::{decorator::bold_text, Printer, NEW_LINE};
use crate::{icon::Icon, model::SubmissionResponse, Either};

#[derive(Debug)]
pub struct SubmitExecutionResult {
    submission_response: SubmissionResponse,
}

impl Printer for SubmitExecutionResult {
    fn is_error(&self) -> bool {
        self.submission_response.is_error()
    }

    fn buffer(&self) -> String {
        if self.is_error() {
            self.error_buffer()
        } else {
            self.success_buffer()
        }
    }
}

impl SubmitExecutionResult {
    pub fn new(submission_response: SubmissionResponse) -> Self {
        Self {
            submission_response,
        }
    }

    fn error_buffer(&self) -> String {
        let error_buffer = self.runtime_error_buffer()
            + NEW_LINE
            + NEW_LINE
            + self.compile_error_buffer().as_str();
        if error_buffer.trim().is_empty() {
            self.wrong_answer_buffer()
        } else {
            error_buffer
        }
    }

    fn runtime_error_buffer(&self) -> String {
        if !self.submission_response.has_runtime_error() {
            return NEW_LINE.to_owned();
        }
        self.submission_response.status_msg.to_owned()
    }

    fn compile_error_buffer(&self) -> String {
        if !self.submission_response.has_compile_error() {
            return NEW_LINE.to_owned();
        }
        self.submission_response
            .full_compile_error
            .to_owned()
            .unwrap_or_default()
    }

    fn wrong_answer_buffer(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&bold_text(
            &Color::Red(&format!(
                "\n{} Wrong Answer: ({})\n\n",
                Icon::_No.to_string(),
                self.total_cases_ratio_buffer(&self.submission_response)
            ))
            .make(),
        ));
        buffer.push_str(&self.last_test_case_buffer());
        buffer.push_str(&Color::Red(&self.get_metas()).make());

        buffer
    }

    fn last_test_case_buffer(&self) -> String {
        let mut buffer = String::new();
        match (
            &self.submission_response.input,
            &self.submission_response.code_output,
            &self.submission_response.expected_output,
        ) {
            (
                Some(Either::String(input)),
                Some(Either::String(ans)),
                Some(Either::String(exp_ans)),
            ) => {
                let mut test_case = String::new();
                test_case.push_str(&Color::Red("Last test case:\n").make());
                test_case.push_str(&format!(
                    "\tInput: \n\t\t{}\n",
                    input.replace('\n', "\n\t\t")
                ));
                test_case.push_str(&format!("\n\tOutput: {}\n", ans));
                test_case.push_str(&format!("\tExpected: {}\n\n", exp_ans));

                buffer.push_str(test_case.as_str());
            }
            _ => {}
        }

        buffer
    }

    fn success_buffer(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str(&bold_text(
            &Color::Green(&format!(
                "{} Accepted: ({})\n\n",
                Icon::Yes.to_string(),
                self.total_cases_ratio_buffer(&self.submission_response)
            ))
            .make(),
        ));
        buffer.push_str(&self.last_test_case_buffer());
        buffer.push_str(&Color::Green(&self.get_metas()).make());

        buffer
    }

    fn get_metas(&self) -> String {
        if self.is_error() {
            return NEW_LINE.to_string();
        }
        let memory_percentile = self
            .submission_response
            .memory_percentile
            .unwrap_or(0.0)
            .to_string();
        let runtime_percentile = self
            .submission_response
            .runtime_percentile
            .unwrap_or(0.0)
            .to_string();
        let metas = vec![
            "Memory: ".to_string() + self.submission_response.status_memory.as_str(),
            "Memory %ile: ".to_string() + memory_percentile.as_str(),
            "Runtime: ".to_string() + self.submission_response.status_runtime.as_str(),
            "Runtime %ile: ".to_string() + runtime_percentile.as_str(),
            "\n\n".to_string(),
        ];

        metas.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{Printer, SubmitExecutionResult};
    use crate::model::SubmissionResponse;
    use serde_json::from_value;

    #[test]
    fn print_submit_wrong_answer() {
        let json_value = serde_json::from_str(
            r#"{
	"status_code": 11,
	"lang": "python3",
	"run_success": true,
	"status_runtime": "N/A",
	"memory": 16468000,
	"question_id": "10",
	"elapsed_time": 65,
	"compare_result": "1110011111111111100011110100100011010111111111111111111101110111111111111111111111110111111110100111010111111111111111011101111111110011111111111111111110101101110010111011011101111101111111110111101011101111101111111111111101101110110111101110011111101001111110110110101101110011001111111111111111001110000010110111111111111110111110111110011111001001010",
	"code_output": "false",
	"std_output": "",
	"last_testcase": "\"aab\"\n\"c*a*b\"",
	"expected_output": "true",
	"task_finish_time": 1694277425292,
	"task_name": "judger.judgetask.Judge",
	"finished": true,
	"total_correct": 277,
	"total_testcases": 355,
	"runtime_percentile": null,
	"status_memory": "N/A",
	"memory_percentile": null,
	"pretty_lang": "Python3",
	"submission_id": "1044868319",
	"input_formatted": "\"aab\", \"c*a*b\"",
	"input": "\"aab\"\n\"c*a*b\"",
	"status_msg": "Wrong Answer",
	"state": "SUCCESS"
}"#
                    )
        .unwrap();

        let response = from_value::<SubmissionResponse>(json_value).unwrap().into();

        let result = SubmitExecutionResult::new(response);
        result.print();
        // TODO implement snapshot testing
        assert!(1 == 1);
    }

    #[test]
    fn print_submit_accepted() {
        let json_value = serde_json::from_str(
            r#"{
	"status_code": 10,
	"lang": "rust",
	"run_success": true,
	"status_runtime": "0 ms",
	"memory": 1928000,
	"question_id": "10",
	"elapsed_time": 10,
	"compare_result": "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
	"code_output": "",
	"std_output": "",
	"last_testcase": "",
	"expected_output": "",
	"task_finish_time": 1694280355711,
	"task_name": "judger.judgetask.Judge",
	"finished": true,
	"total_correct": 355,
	"total_testcases": 355,
	"runtime_percentile": 100,
	"status_memory": "1.9 MB",
	"memory_percentile": 91.83670000000001,
	"pretty_lang": "Rust",
	"submission_id": "1044907055",
	"status_msg": "Accepted",
	"state": "SUCCESS"
}"#
        )
        .unwrap();

        let response = from_value::<SubmissionResponse>(json_value).unwrap().into();

        let result = SubmitExecutionResult::new(response);
        result.print();
        // TODO implement snapshot testing
        assert!(1 == 1);
    }

    #[test]
    fn print_submit_wrong_answer_seq() {
        let json_value = serde_json::from_str(
r#"{
	"status_code": 11,
	"lang": "java",
	"run_success": true,
	"status_runtime": "N/A",
	"memory": 44740000,
	"display_runtime": "261",
	"question_id": "15",
	"elapsed_time": 467,
	"compare_result": "111111111111111111110010011100110000111100001101000100000010111101100110011101001001000000010101101000000000000001111100001000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100",
	"code_output": "[[-2,-1,3],[-2,0,2]]",
	"std_output": "",
	"last_testcase": "[3,0,-2,-1,1,2]",
	"expected_output": "[[-2,-1,3],[-2,0,2],[-1,0,1]]",
	"task_finish_time": 1694280912080,
	"task_name": "judger.judgetask.Judge",
	"finished": true,
	"total_correct": 63,
	"total_testcases": 312,
	"runtime_percentile": null,
	"status_memory": "N/A",
	"memory_percentile": null,
	"pretty_lang": "Java",
	"submission_id": "1044914848",
	"input_formatted": "[3,0,-2,-1,1,2]",
	"input": "[3,0,-2,-1,1,2]",
	"status_msg": "Wrong Answer",
	"state": "SUCCESS"
}"#
        )
        .unwrap();

        let response = from_value::<SubmissionResponse>(json_value).unwrap().into();

        let result = SubmitExecutionResult::new(response);
        result.print();
        // TODO implement snapshot testing
        assert!(1 == 1);
    }
}
