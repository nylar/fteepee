use crate::response::Addr;

use nom::{
    bytes::complete::{take, take_until, take_while_m_n},
    character::is_digit,
    combinator::map_res,
    sequence::tuple,
    Finish, IResult,
};

use crate::Result;

#[cfg(feature = "std")]
pub fn parse_features(buf: &[u8]) -> std::collections::HashMap<&str, Option<&str>> {
    let lines = buf.split(|byte| *byte == b'\n');

    lines
        .filter_map(|line| match line {
            [b' ', rest @ ..] => Some(rest),
            _ => None,
        })
        .filter_map(|line| {
            let mut group = line.splitn(2, |byte| *byte == b' ');

            match (group.next(), group.next()) {
                (Some(feature), Some(details)) => Some((
                    std::str::from_utf8(feature).expect("Invalid UTF-8").trim(),
                    Some(std::str::from_utf8(details).expect("Invalid UTF-8").trim()),
                )),
                (Some(feature), None) => Some((
                    std::str::from_utf8(feature).expect("Invalid UTF-8").trim(),
                    None,
                )),
                _ => None,
            }
        })
        .collect::<std::collections::HashMap<_, _>>()
}

pub fn parse_passive_mode(buf: &[u8]) -> Result<Addr> {
    let (_, (_, _, first, _, second, _, third, _, fourth, _, msb, _, lsb, _)) = tuple((
        take_until("("),
        take(1usize),
        to_u8,
        take(1usize),
        to_u8,
        take(1usize),
        to_u8,
        take(1usize),
        to_u8,
        take(1usize),
        to_u16,
        take(1usize),
        to_u16,
        take(1usize),
    ))(buf)
    .finish()
    .unwrap();

    Ok(([first, second, third, fourth], (msb << 8) + lsb))
}

fn to_u8(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(take_while_m_n(1, 3, is_digit), btoi::btou)(input)
}

fn to_u16(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(take_while_m_n(1, 3, is_digit), btoi::btou)(input)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_features() {
        let buf = b"Extensions supported:\n EPRT\n IDLE\n MDTM\n SIZE\n MFMT\n REST STREAM\n MLST type*;size*;sizd*;modify*;UNIX.mode*;UNIX.uid*;UNIX.gid*;unique*;\n MLSD\n AUTH TLS\n PBSZ\n PROT\n UTF8\n TVFS\n ESTA\n PASV\n EPSV\n SPSV\r\nEnd.";

        let feats = super::parse_features(&buf[..]);

        let expected = HashMap::from([
            ("MFMT", None),
            ("EPSV", None),
            ("PASV", None),
            ("ESTA", None),
            ("AUTH", Some("TLS")),
            ("REST", Some("STREAM")),
            ("MLSD", None),
            ("IDLE", None),
            ("TVFS", None),
            ("SPSV", None),
            ("EPRT", None),
            ("PBSZ", None),
            ("MDTM", None),
            (
                "MLST",
                Some("type*;size*;sizd*;modify*;UNIX.mode*;UNIX.uid*;UNIX.gid*;unique*;"),
            ),
            ("PROT", None),
            ("UTF8", None),
            ("SIZE", None),
        ]);

        assert_eq!(expected, feats);
    }

    #[test]
    #[should_panic(expected = "Invalid UTF-8")]
    fn test_invalid_utf8() {
        super::parse_features(b" \xfe \xff");
    }
}
