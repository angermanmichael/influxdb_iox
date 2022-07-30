use nom::{bytes::complete::take_until, IResult};

#[derive(Debug)]
pub enum NuIoxErrorType {
    TableNotFound,
}

#[derive(Debug)]
pub struct NuIoxError {
    #[allow(dead_code)]
    start: String,
    error_type: NuIoxErrorType,
    header: String,
    status: String,
    message: String,
}

impl NuIoxError {
    pub fn build(data: &str) -> Self {
        let details = remove_details(data).unwrap().1;
        let (message0, remainder) = get_message(details).unwrap();
        let (status0, header0) = get_header(&remainder).unwrap();

        let header1 = remove_colon_from_string(&header0.to_string());
        let message1 = remove_slash_from_string(&message0.to_string());

        Self {
            start: data.to_string(),
            error_type: NuIoxErrorType::TableNotFound,
            header: header1,
            status: status0.to_string(),
            message: message1,
        }
    }

    pub fn print(self) {
        //println!("{:?}", self.start.trim());
        println!("{:?}", self.error_type);
        println!("{:?}", self.header.trim());
        println!("{:?}", self.status.trim());
        println!("{:?}", self.message.trim());
    }
}

fn remove_details(s: &str) -> IResult<&str, &str> {
    let details: &'static str = ", details: ";
    take_until(details)(s)
}

fn get_message(s: &str) -> IResult<&str, &str> {
    let msg: &'static str = ", message: ";
    take_until(msg)(s)
}

fn get_header(s: &str) -> IResult<&str, &str> {
    let header: &'static str = "status: ";
    take_until(header)(s)
}

fn remove_slash_from_string(s: &String) -> String {
    s.replace(&['(', ')', ',', '\"', ';', '\''][..], "")
}

fn remove_colon_from_string(s: &String) -> String {
    s.replace(&[':'][..], "")
}

use nu_protocol::ast::Call;
use nu_protocol::ShellError;

#[derive(Copy, Clone, Debug)]
pub enum CommandType {
    Sql,
    Write,
    WriteFile,
}

pub struct NuIoxErrorHandler {
    #[allow(dead_code)]
    ctype: CommandType,
    error: String,
}

impl NuIoxErrorHandler {
    pub fn new(ctype: CommandType, error: String) -> Self {
        let result = NuIoxError::build(error.as_ref());
        result.print();
        Self { ctype, error }
    }

    // Check and see if its an error or a csv
    pub fn nu_iox_error_check(&self) -> Result<String, ShellError> {
        //println!("{:?}", self.error);
        Ok(self.error.clone())
    }

    // Trigger an error to see what the Error looks like
    pub fn nu_iox_error_generic(
        &self,
        str01: &str,
        str02: &str,
        call: &Call,
    ) -> Result<String, ShellError> {
        return Err(ShellError::GenericError(
            str01.to_string(),
            str02.to_string(),
            Some(call.head),
            None,
            Vec::new(),
        ));
    }
}
