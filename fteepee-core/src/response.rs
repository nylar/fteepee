use crate::{parsers::parse_passive_mode, Code, Error, Result};

pub type Addr = ([u8; 4], u16);

const NEWLINE: &[u8] = b"\n";

#[derive(Clone, Copy, Debug)]
pub enum ParsedResponseState {
    Empty,
    Partial,
    Complete,
}

impl Default for ParsedResponseState {
    fn default() -> Self {
        Self::Empty
    }
}

pub struct Response {
    code: [u8; 3],
    state: ParsedResponseState,
    cursor: usize,
}

impl Response {
    pub fn new() -> Self {
        Self {
            code: [0; 3],
            state: ParsedResponseState::default(),
            cursor: 0,
        }
    }

    pub fn code(&self) -> Result<Code> {
        Code::try_from(self.code)
    }

    pub fn message<'a>(&self, input: &'a [u8]) -> &'a [u8] {
        &input[..self.cursor]
    }

    pub fn read_bytes(
        &mut self,
        input: &[u8],
        mut output: &mut [u8],
    ) -> Result<ParsedResponseState> {
        match self.state {
            ParsedResponseState::Empty => match input {
                [f @ b'0'..=b'9', s @ b'0'..=b'9', t @ b'0'..=b'9', multiline @ (b' ' | b'-'), rest @ ..] =>
                {
                    self.code = [*f, *s, *t];
                    self.cursor += output.append_bytes(rest, self.cursor);

                    if *multiline == b'-' {
                        self.cursor += output.append_bytes(NEWLINE, self.cursor);
                        self.state = ParsedResponseState::Partial;
                    } else {
                        self.state = ParsedResponseState::Complete;
                    }
                }
                _ => return Err(Error::IncompleteResponse),
            },
            ParsedResponseState::Partial => {
                // TODO: Should we detect incorrect codes here?
                // 220-
                // 210 # expected 220
                match input {
                    [f @ b'0'..=b'9', s @ b'0'..=b'9', t @ b'0'..=b'9', b' ', rest @ ..]
                        if [*f, *s, *t] == self.code =>
                    {
                        self.cursor += output.append_bytes(rest, self.cursor);
                        self.state = ParsedResponseState::Complete;
                    }
                    [f @ b'0'..=b'9', s @ b'0'..=b'9', t @ b'0'..=b'9', b'-', rest @ ..]
                        if [*f, *s, *t] == self.code =>
                    {
                        self.cursor += output.append_bytes(rest, self.cursor);
                        self.cursor += output.append_bytes(NEWLINE, self.cursor);
                    }
                    _ => {
                        self.cursor += output.append_bytes(input, self.cursor);
                        self.cursor += output.append_bytes(NEWLINE, self.cursor);
                    }
                }
            }
            ParsedResponseState::Complete => {}
        }

        Ok(self.state)
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseExt for Response {
    fn parse_passive_mode(&mut self, input: &[u8]) -> Result<Addr> {
        let addr = parse_passive_mode(input)?;

        Ok(addr)
    }
}

pub trait ResponseExt {
    fn parse_passive_mode(&mut self, input: &[u8]) -> Result<Addr>;
}

trait AppendBytes {
    fn append_bytes(&mut self, buf: &[u8], cursor: usize) -> usize;
}

impl AppendBytes for &mut [u8] {
    fn append_bytes(&mut self, buf: &[u8], cursor: usize) -> usize {
        // Trim CRLF
        let b = match buf {
            [rest @ .., b'\r', b'\n'] => rest,
            _ => buf,
        };

        self[cursor..cursor + b.len()].copy_from_slice(b);
        b.len()
    }
}

#[cfg(test)]
mod tests {
    use core::io::{BufRead, BufReader};

    use crate::{
        response::{ParsedResponseState, Response, ResponseExt},
        Code, Result,
    };

    #[test]
    fn test_parses_individual_command_successfully() {
        let mut buf: [u8; 4096] = [0; 4096];

        let resp = parse_response(
            include_bytes!("../testdata/individual_command_input"),
            &mut buf,
        )
        .unwrap();

        assert_eq!(resp.code().unwrap(), Code::CREATED);
        assert_eq!(
            resp.message(&buf),
            stripped(include_bytes!("../testdata/individual_command_output"))
        );
    }

    #[test]
    fn test_parses_double_command_successfully() {
        let mut buf: [u8; 4096] = [0; 4096];

        let resp =
            parse_response(include_bytes!("../testdata/double_command_input"), &mut buf).unwrap();

        assert_eq!(resp.code().unwrap(), Code::CLOSING_CONTROL_CONNECTION);
        assert_eq!(
            resp.message(&buf),
            stripped(include_bytes!("../testdata/double_command_output"))
        );
    }

    #[test]
    fn test_parses_welcome_message_successfully() {
        let mut buf: [u8; 4096] = [0; 4096];

        let resp = parse_response(
            include_bytes!("../testdata/welcome_message_input"),
            &mut buf,
        )
        .unwrap();

        assert_eq!(resp.code().unwrap(), Code::READY);
        assert_eq!(
            resp.message(&buf),
            stripped(include_bytes!("../testdata/welcome_message_output"))
        );
    }

    #[test]
    fn test_parses_feat_successfully() {
        let mut buf: [u8; 4096] = [0; 4096];

        let resp =
            parse_response(include_bytes!("../testdata/feat_command_input"), &mut buf).unwrap();

        assert_eq!(resp.code().unwrap(), Code::STATUS);
        assert_eq!(
            resp.message(&buf),
            stripped(include_bytes!("../testdata/feat_command_output"))
        );
    }

    #[test]
    fn test_response_parse_passive_mode() {
        let mut resp = Response::new();

        let addr = ([127, 0, 0, 1], 30001);

        assert_eq!(
            resp.parse_passive_mode(b"Entering Passive Mode (127,0,0,1,117,49)")
                .unwrap(),
            addr
        );
    }

    fn parse_response(buf: &[u8], response_buf: &mut [u8]) -> Result<Response> {
        let r = BufReader::new(buf);

        let mut lines = r.lines();

        let mut parsed_response = Response::new();

        while let ParsedResponseState::Partial =
            parsed_response.read_bytes(lines.next().unwrap().unwrap().as_bytes(), response_buf)?
        {
        }

        Ok(parsed_response)
    }

    fn stripped(buf: &[u8]) -> &[u8] {
        match buf {
            [.., b'\n'] => &buf[..buf.len() - 1],
            _ => buf,
        }
    }
}
