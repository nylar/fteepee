const TELNET_END_OF_LINE: &[u8] = b"\r\n";
const CMD: usize = 4;
const SPACE: usize = 1;
const EOL: usize = TELNET_END_OF_LINE.len();

fn write(buf: &mut [u8], data: &[u8], n: usize) -> usize {
    buf[n..n + data.len()].copy_from_slice(data);
    data.len()
}

macro_rules! impl_commands {
	(
		$(
			($t:ty, $cmd:literal $(, $field:ident)*);
		)+
	) => {
		$(
			impl Command for $t {
				fn encode(&self, buf: &mut [u8]) {
					let mut n = 0usize;

					n += write(buf, &$cmd[..], n);
					$(
						n += write(buf, b" ", n);
						n += write(buf, self.$field.as_bytes(), n);
					)*
					write(buf, TELNET_END_OF_LINE, n);
				}

				fn size(&self) -> usize {
					CMD + $(SPACE + self.$field.len() +)* EOL
				}
			}
		)*
	}
}

pub trait Command {
    fn encode(&self, buf: &mut [u8]);

    fn size(&self) -> usize;
}

pub enum Type {
    ASCII,
    EBCDIC,
    Image,
    Local,
}

impl_commands! {
    (User<'_>, b"USER", user);
    (Pass<'_>, b"PASS", pass);
    (List<'_>, b"LIST", path);
    (Mlsd<'_>, b"MLSD", path);
    (Pasv, b"PASV");
    (Syst, b"SYST");
    (Feat, b"FEAT");
    (Stor<'_>, b"STOR", path);
}

impl Command for Type {
    fn encode(&self, buf: &mut [u8]) {
        let ty = match *self {
            Type::ASCII => b"A",
            Type::EBCDIC => b"E",
            Type::Image => b"I",
            Type::Local => b"L",
        };

        let mut n = 0usize;

        n += write(buf, &b"TYPE"[..], n);
        n += write(buf, &b" "[..], n);
        n += write(buf, &ty[..], n);
        write(buf, TELNET_END_OF_LINE, n);
    }

    fn size(&self) -> usize {
        CMD + SPACE + 1 + EOL
    }
}

pub struct User<'a> {
    user: &'a str,
}

impl<'a> User<'a> {
    pub fn new(user: &'a str) -> Self {
        Self { user }
    }
}

pub struct Pass<'a> {
    pass: &'a str,
}

impl<'a> Pass<'a> {
    pub fn new(pass: &'a str) -> Self {
        Self { pass }
    }
}

pub struct List<'a> {
    path: &'a str,
}

impl<'a> List<'a> {
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }
}

pub struct Mlsd<'a> {
    path: &'a str,
}

impl<'a> Mlsd<'a> {
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }
}

#[derive(Default)]
pub struct Pasv;

#[derive(Default)]
pub struct Syst;

#[derive(Default)]
pub struct Feat;

pub struct Stor<'a> {
    path: &'a str,
}

impl<'a> Stor<'a> {
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_command() {
        let mut output: [u8; 64] = [0; 64];

        let cmd = Type::ASCII;

        cmd.encode(&mut output);

        assert_eq!(output[..cmd.size()], b"TYPE A\r\n"[..]);
    }

    #[test]
    fn empty_struct_command() {
        let mut output: [u8; 64] = [0; 64];

        let cmd = Pasv::default();

        cmd.encode(&mut output);

        assert_eq!(output[..cmd.size()], b"PASV\r\n"[..]);
    }

    #[test]
    fn one_field_struct_command() {
        let mut output: [u8; 64] = [0; 64];

        let cmd = User::new("foo");

        cmd.encode(&mut output);

        assert_eq!(output[..cmd.size()], b"USER foo\r\n"[..]);
    }
}
