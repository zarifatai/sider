// Finished up and till 2.10

use crate::resp_result::{RESPError, RESPResult};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RESP {
    SimpleString(String),
}

impl fmt::Display for RESP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            Self::SimpleString(data) => format!("+{}\r\n", data),
        };

        write!(f, "{}", data)
    }
}

fn binary_extract_line(buffer: &[u8], index: &mut usize) -> RESPResult<Vec<u8>> {
    if *index >= buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }

    let start_index = *index;
    let mut current_char = &buffer[start_index];
    let mut seperator_found = false;
    for char in buffer[start_index + 1..].iter() {
        *index += 1;
        if *current_char == b'\r' && *char == b'\n' {
            seperator_found = true;
            break;
        }
        current_char = char;
    }

    *index += 1;
    if !seperator_found {
        return Err(RESPError::OutOfBounds(*index));
    }

    Ok(Vec::from(&buffer[start_index..*index - 2]))
}

pub fn binary_extract_line_as_string(buffer: &[u8], index: &mut usize) -> RESPResult<String> {
    let line = binary_extract_line(buffer, index)?;

    Ok(String::from_utf8(line)?)
}

pub fn resp_remove_type(value: char, buffer: &[u8], index: &mut usize) -> RESPResult<()> {
    if buffer[*index] != value as u8 {
        return Err(RESPError::WrongType);
    }

    *index += 1;

    Ok(())
}

fn parse_simple_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('+', buffer, index)?;

    let line: String = binary_extract_line_as_string(buffer, index)?;

    Ok(RESP::SimpleString(line))
}

fn parser_router(
    buffer: &[u8],
    index: &mut usize,
) -> Option<fn(&[u8], &mut usize) -> RESPResult<RESP>> {
    match buffer[*index] {
        b'+' => Some(parse_simple_string),
        _ => None,
    }
}

pub fn bytes_to_resp(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    match parser_router(buffer, index) {
        Some(parse_func) => {
            let result: RESP = parse_func(buffer, index)?;
            Ok(result)
        }
        None => Err(RESPError::Unknown),
    }
}

fn binary_extract_bytes(buffer: &[u8], index: &mut usize, length: usize) -> RESPResult<Vec<u8>> {
    let mut output = Vec::new();

    if *index + length > buffer.len() {
        return Err(RESPError::OutOfBounds(buffer.len()));
    }

    output.extend_from_slice(&buffer[*index..*index + length]);

    *index += length;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::resp_result::RESPError;

    use super::*;

    #[test]
    fn test_binary_extract_line() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line(buffer, &mut index).unwrap();

        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_extract_line_longer_string() {
        let buffer = "ECHO\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line(buffer, &mut index).unwrap();

        assert_eq!(output, "ECHO".as_bytes());
        assert_eq!(index, 6);
    }

    #[test]
    fn test_binary_extract_line_empty_buffer() {
        let buffer = "".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_no_seperator() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_index_too_advanced() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 1;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_half_separator() {
        let buffer = "OK\r".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_incorrect_separator() {
        let buffer = "OK\n".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_binary_extract_line_as_string() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line_as_string(buffer, &mut index).unwrap();

        assert_eq!(output, String::from("OK"));
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_extact_line_as_string_invalid_utf8() {
        let buffer: Vec<u8> = vec![0xFF, 0xFE, b'\r', b'\n'];
        let mut index: usize = 0;

        let error = binary_extract_line_as_string(&buffer, &mut index).unwrap_err();

        assert_eq!(error, RESPError::FromUtf8);
    }

    #[test]
    fn test_resp_remove_type() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        resp_remove_type('+', buffer, &mut index).unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn test_resp_remove_type_error() {
        let buffer = "*OK\r\n".as_bytes();
        let mut index: usize = 0;

        let error = resp_remove_type('+', buffer, &mut index).unwrap_err();

        assert_eq!(index, 0);
        assert_eq!(error, RESPError::WrongType);
    }

    #[test]
    fn test_parse_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = parse_simple_string(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }

    #[test]
    fn test_bytes_to_resp_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = bytes_to_resp(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }

    #[test]
    fn test_byte_to_resp_unknown() {
        let buffer = "?OK\r\n".as_bytes();
        let mut index: usize = 0;

        let error = bytes_to_resp(buffer, &mut index).unwrap_err();

        assert_eq!(error, RESPError::Unknown);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_binary_extract_bytes() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_bytes(buffer, &mut index, 6).unwrap();

        assert_eq!(output, "SOMEBY".as_bytes().to_vec());
        assert_eq!(index, 6);
    }

    #[test]
    fn test_binary_extract_bytes_out_of_bounds() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;

        let error = binary_extract_bytes(buffer, &mut index, 10).unwrap_err();

        assert_eq!(error, RESPError::OutOfBounds(9));
        assert_eq!(index, 0)
    }
}
