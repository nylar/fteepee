use std::{
    error, fmt,
    io::{BufRead, BufReader, Write},
    marker::PhantomData,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use bytes::BytesMut;
use fteepee_core::{
    commands::{Command, Feat, List, Mlsd, Pass, Pasv, User},
    expect_code,
    response::{ParsedResponseState, Response, ResponseExt},
    Code, Config, Connected, Disconnected,
};

pub struct Client<State = Disconnected> {
    stream: Lines<BufReader<TcpStream>>,
    read_buffer: Vec<u8>,
    write_buffer: BytesMut,
    data_buffer: Vec<u8>,
    response_buffer: Vec<u8>,
    marker: PhantomData<State>,
    config: Config,
}

impl Client<Disconnected> {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Client<Connected>> {
        let stream = TcpStream::connect(addr)?;

        let mut client = Client {
            stream: Lines::new(BufReader::new(stream)),
            read_buffer: Vec::new(),
            write_buffer: BytesMut::new(),
            data_buffer: Vec::new(),
            response_buffer: Vec::new(),
            marker: PhantomData,
            config: Config::default(),
        };

        let resp = client.read_response()?;
        expect_code!(resp.code()?, Code::READY);

        let cmd = Feat::default();
        client.write_request(&cmd)?;
        let resp = client.read_response()?;
        expect_code!(
            resp.code()?,
            Code::STATUS | Code::UNRECOGNIZED_COMMAND | Code::NOT_IMPLEMENTED
        );

        if matches!(resp.code()?, Code::STATUS) {
            let features = fteepee_core::parse_features(&client.response_buffer);

            if features.get("MLST").is_some() {
                client.config.mlst_supported = true;
            }
        };

        Ok(client)
    }
}

impl Client<Connected> {
    pub fn login(&mut self, user: &str, pass: &str) -> Result<()> {
        let cmd = User::new(user);

        self.write_request(&cmd)?;
        let resp = self.read_response()?;
        expect_code!(resp.code()?, Code::LOGGED_IN | Code::PASSWORD_REQUIRED);

        let cmd = Pass::new(pass);

        self.write_request(&cmd)?;
        let resp = self.read_response()?;
        expect_code!(resp.code()?, Code::LOGGED_IN);

        Ok(())
    }

    pub fn list(&mut self, path: &str) -> Result<()> {
        let stream = if self.config.mlst_supported {
            let cmd = Mlsd::new(path);
            BufReader::new(self.data_connection(&cmd)?)
        } else {
            let cmd = List::new(path);
            BufReader::new(self.data_connection(&cmd)?)
        };

        let resp = self.read_response()?;
        expect_code!(resp.code()?, Code::OPENING_DATA_CONNECTION);

        let mut lines = Lines::new(stream);

        while let Some(line) = lines.next(&mut self.data_buffer) {
            let line = line?;

            dbg!(String::from_utf8_lossy(line));
        }

        let resp = self.read_response()?;
        expect_code!(
            resp.code()?,
            Code::CLOSING_DATA_CONNECTION | Code::REQUESTED_FILE_ACTION_OKAY,
        );

        Ok(())
    }

    fn data_connection<C: Command>(&mut self, cmd: &C) -> Result<TcpStream> {
        let addr = self.pasv()?;

        self.write_request(cmd)?;

        let stream = TcpStream::connect(addr)?;

        Ok(stream)
    }

    fn pasv(&mut self) -> Result<SocketAddr> {
        let cmd = Pasv::default();

        self.write_request(&cmd)?;
        let mut resp = self.read_response()?;
        expect_code!(resp.code()?, Code::ENTERING_PASSIVE_MODE);

        Ok(SocketAddr::from(
            resp.parse_passive_mode(&self.response_buffer)?,
        ))
    }

    fn read_response(&mut self) -> Result<Response> {
        self.read_buffer.clear();

        let mut parsed_response = Response::new();

        while let Some(line) = self.stream.next(&mut self.read_buffer) {
            let line = line?;
            self.response_buffer
                .resize(self.response_buffer.len() + line.len(), 0);

            let state = parsed_response.read_bytes(line, &mut self.response_buffer)?;

            // FIXME: I will block forever if I don't reach a completed state
            if matches!(state, ParsedResponseState::Complete) {
                break;
            }
        }

        Ok(parsed_response)
    }

    fn write_request<C: Command>(&mut self, cmd: &C) -> Result<()> {
        self.write_buffer.clear();

        self.write_buffer.resize(cmd.size(), 0);

        cmd.encode(&mut self.write_buffer);

        self.stream
            .reader
            .get_mut()
            .write_all(&self.write_buffer[..cmd.size()])?;

        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Internal(fteepee_core::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::Internal(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::Internal(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<fteepee_core::Error> for Error {
    fn from(err: fteepee_core::Error) -> Self {
        Self::Internal(err)
    }
}

struct Lines<B: BufRead> {
    reader: B,
}

impl<B: BufRead> Lines<B> {
    fn new(reader: B) -> Self {
        Self { reader }
    }

    fn next<'a>(&mut self, buf: &'a mut Vec<u8>) -> Option<Result<&'a [u8]>> {
        buf.clear();

        match self.reader.read_until(b'\n', buf) {
            Ok(0) => None,
            Ok(n) => {
                let bytes = match buf[..] {
                    [.., b'\r', b'\n'] => &buf[..n - 2],
                    [.., b'\n'] => &buf[..n - 1],
                    _ => &buf[..n],
                };
                Some(Ok(bytes))
            }
            Err(e) => Some(Err(Error::IO(e))),
        }
    }
}
