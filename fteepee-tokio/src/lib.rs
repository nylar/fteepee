use std::{error, fmt, marker::PhantomData, net::SocketAddr};

use bytes::BytesMut;
use fteepee_core::{
    commands::{Command, Feat, List, Mlsd, Pass, Pasv, Stor, Type, User},
    expect_code,
    response::{ParsedResponseState, Response, ResponseExt},
    Code, Config, Connected, Disconnected,
};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpStream, ToSocketAddrs},
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
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Client<Connected>> {
        let stream = TcpStream::connect(addr).await?;

        let mut client = Client {
            stream: Lines::new(BufReader::new(stream)),
            read_buffer: Vec::new(),
            write_buffer: BytesMut::new(),
            data_buffer: Vec::new(),
            response_buffer: Vec::new(),
            marker: PhantomData,
            config: Config::default(),
        };

        let resp = client.read_response().await?;
        expect_code!(resp.code()?, Code::READY);

        let cmd = Feat::default();
        client.write_request(&cmd).await?;
        let resp = client.read_response().await?;
        expect_code!(
            resp.code()?,
            Code::SYSTEM_STATUS | Code::UNRECOGNIZED_COMMAND | Code::NOT_IMPLEMENTED
        );

        if matches!(resp.code()?, Code::SYSTEM_STATUS) {
            let features = fteepee_core::parse_features(&client.response_buffer);

            if features.get("MLST").is_some() {
                client.config.mlst_supported = true;
            }
        };

        Ok(client)
    }
}

impl Client<Connected> {
    pub async fn login(&mut self, user: &str, pass: &str) -> Result<()> {
        let cmd = User::new(user);

        self.write_request(&cmd).await?;
        let resp = self.read_response().await?;
        expect_code!(resp.code()?, Code::LOGGED_IN | Code::PASSWORD_REQUIRED);

        let cmd = Pass::new(pass);

        self.write_request(&cmd).await?;
        let resp = self.read_response().await?;
        expect_code!(resp.code()?, Code::LOGGED_IN);

        Ok(())
    }

    pub async fn list(&mut self, path: &str) -> Result<()> {
        let stream = if self.config.mlst_supported {
            let cmd = Mlsd::new(path);
            BufReader::new(self.data_connection(&cmd).await?)
        } else {
            let cmd = List::new(path);
            BufReader::new(self.data_connection(&cmd).await?)
        };

        let resp = self.read_response().await?;
        expect_code!(resp.code()?, Code::OPENING_DATA_CONNECTION);

        let mut lines = Lines::new(stream);

        while let Some(line) = lines.next(&mut self.data_buffer).await {
            let line = line?;

            dbg!(String::from_utf8_lossy(line));
        }

        let resp = self.read_response().await?;
        expect_code!(
            resp.code()?,
            Code::CLOSING_DATA_CONNECTION | Code::REQUESTED_FILE_ACTION_OKAY,
        );

        Ok(())
    }

    pub async fn put<R: AsyncRead + Unpin + ?Sized>(
        &mut self,
        path: &str,
        reader: &mut R,
    ) -> Result<()> {
        let cmd = Type::Image;

        self.write_request(&cmd).await?;
        let resp = self.read_response().await?;
        expect_code!(resp.code()?, Code::COMMAND_OKAY);

        let cmd = Stor::new(path);
        let mut stream = BufWriter::new(self.data_connection(&cmd).await?);

        let resp = self.read_response().await?;
        expect_code!(
            resp.code()?,
            Code::DATA_CONNECTION_ALREADY_OPEN | Code::OPENING_DATA_CONNECTION
        );

        tokio::io::copy(reader, &mut stream).await?;

        // We are done with this connection
        drop(stream);

        let resp = self.read_response().await?;
        expect_code!(
            resp.code()?,
            Code::CLOSING_DATA_CONNECTION | Code::REQUESTED_FILE_ACTION_OKAY,
        );

        Ok(())
    }

    async fn data_connection<C: Command>(&mut self, cmd: &C) -> Result<TcpStream> {
        let addr = self.pasv().await?;

        self.write_request(cmd).await?;

        let stream = TcpStream::connect(addr).await?;

        Ok(stream)
    }

    async fn pasv(&mut self) -> Result<SocketAddr> {
        let cmd = Pasv::default();

        self.write_request(&cmd).await?;
        let mut resp = self.read_response().await?;
        expect_code!(resp.code()?, Code::ENTERING_PASSIVE_MODE);

        Ok(SocketAddr::from(
            resp.parse_passive_mode(&self.response_buffer)?,
        ))
    }

    async fn read_response(&'_ mut self) -> Result<Response> {
        self.read_buffer.clear();

        let mut parsed_response = Response::new();

        while let Some(line) = self.stream.next(&mut self.read_buffer).await {
            let line = line?;
            self.response_buffer
                .resize(self.response_buffer.len() + line.len(), 0);

            let state = parsed_response.read_bytes(line, &mut self.response_buffer)?;

            if matches!(state, ParsedResponseState::Complete) {
                break;
            }
        }

        Ok(parsed_response)
    }

    async fn write_request<C: Command>(&mut self, cmd: &C) -> Result<()> {
        self.write_buffer.clear();

        self.write_buffer.resize(cmd.size(), 0);

        cmd.encode(&mut self.write_buffer);

        self.stream
            .reader
            .get_mut()
            .write_all(&self.write_buffer[..cmd.size()])
            .await?;

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

struct Lines<B: AsyncBufRead + Unpin> {
    reader: B,
}

impl<B: AsyncBufRead + Unpin> Lines<B> {
    fn new(reader: B) -> Self {
        Self { reader }
    }

    async fn next<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> Option<Result<&'a [u8]>> {
        buf.clear();

        match self.reader.read_until(b'\n', buf).await {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
