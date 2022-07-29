use nu_protocol::ast::Call;
use nu_protocol::ShellError;

#[derive(Copy, Clone, Debug)]
pub enum CommandType {
    Sql,
    Write,
    WriteFile,
}

// #[derive(Clone, Debug)]
pub struct NuIoxErrorHandler {
    #[allow(dead_code)]
    ctype: CommandType,
    error: String,
}

impl NuIoxErrorHandler {
    pub fn new(ctype: CommandType, error: String) -> Self {
        Self { ctype, error }
    }

    // Check and see if its an error or a csv
    pub fn nu_iox_error_check(&self) -> Result<String, ShellError> {
        //println!("{:?}", self.error);
        Ok(self.error.clone())
    }

    // Trigger an error to see what the Error looks like
    pub fn nu_iox_error_test(&self, call: &Call) -> Result<String, ShellError> {
        return Err(ShellError::UnsupportedInput(
            "Drop nth accepts only positive integers".to_string(),
            call.head,
        ));
    }
}

use nom::{bytes::complete::is_a, IResult};

pub fn is_a_error(s: &str) -> IResult<&str, &str> {
    let remote_query: &'static str = "Error";
    is_a(remote_query)(s)
}

// This returns true if there is not the word Error in the string,
// meaning that an error was thrown by nom because it can not find the Error string
// This returns false if the string has the word Error in it
pub fn error_check(s: &str) -> bool {
    let result = is_a_error(s);
    println!("error_check result 2 = {:?}", result);

    let mybool = match result.is_err() {
        true => true,
        false => false,
    };

    return mybool;
}
