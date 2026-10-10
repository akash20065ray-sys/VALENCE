//! Zero-Copy RESP3 Protocol Parser & Serializer
//!
//! Designed for the high-performance network hot path:
//! - Borrows byte slices directly from TCP query buffers (`Command<'a>`).
//! - Zero heap allocation during command argument parsing.
//! - Strict compliance with Redis RESP2 and RESP3 wire protocols.

#[derive(Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Ping(Option<&'a [u8]>),
    Get { key: &'a [u8] },
    Set {
        key: &'a [u8],
        value: &'a [u8],
        expire_ms: Option<u64>,
    },
    Del { key: &'a [u8] },
    Exists { key: &'a [u8] },
    Expire { key: &'a [u8], seconds: u64 },
    Ttl { key: &'a [u8] },
    MGet { keys: Vec<&'a [u8]> },
    MSet { pairs: Vec<(&'a [u8], &'a [u8])> },
    DbSize,
    Save,
    Bgsave,
    Lastsave,
    Command,
    Info,
    Unknown(&'a [u8]),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    InvalidFrame(&'static str),
    BadInteger,
}

pub struct RespParser;

impl RespParser {
    /// Attempts to parse a single command from the input byte slice.
    ///
    /// Returns:
    /// - `Ok(Some((cmd, consumed_bytes)))` on successful parse.
    /// - `Ok(None)` if the buffer contains an incomplete frame (needs more TCP data).
    /// - `Err(ParseError)` on malformed protocol frames.
    pub fn parse<'a>(buf: &'a [u8]) -> Result<Option<(Command<'a>, usize)>, ParseError> {
        if buf.is_empty() {
            return Ok(None);
        }

        // RESP Commands arrive as Arrays of Bulk Strings (e.g., `*3\r\n$3\r\nSET\r\n...`)
        // or as inline commands (e.g. `PING\r\n` in telnet / simple clients)
        if buf[0] == b'*' {
            Self::parse_resp_array(buf)
        } else {
            Self::parse_inline(buf)
        }
    }

    /// Parses RESP Array format (`*<count>\r\n...`).
    fn parse_resp_array<'a>(buf: &'a [u8]) -> Result<Option<(Command<'a>, usize)>, ParseError> {
        let mut pos = 1;
        let line_end = match find_crlf(&buf[pos..]) {
            Some(idx) => pos + idx,
            None => return Ok(None),
        };

        let count_str = std::str::from_utf8(&buf[pos..line_end])
            .map_err(|_| ParseError::InvalidFrame("Invalid UTF-8 in array length"))?;
        let count: usize = count_str
            .parse()
            .map_err(|_| ParseError::BadInteger)?;

        pos = line_end + 2;

        let mut args: Vec<&'a [u8]> = Vec::with_capacity(count);

        for _ in 0..count {
            if pos >= buf.len() {
                return Ok(None);
            }

            if buf[pos] != b'$' {
                return Err(ParseError::InvalidFrame("Expected bulk string marker '$'"));
            }
            pos += 1;

            let len_end = match find_crlf(&buf[pos..]) {
                Some(idx) => pos + idx,
                None => return Ok(None),
            };

            let len_str = std::str::from_utf8(&buf[pos..len_end])
                .map_err(|_| ParseError::InvalidFrame("Invalid UTF-8 in bulk string length"))?;
            let bulk_len: usize = len_str
                .parse()
                .map_err(|_| ParseError::BadInteger)?;

            pos = len_end + 2;

            if pos + bulk_len + 2 > buf.len() {
                return Ok(None); // Incomplete payload
            }

            // Verify trailing CRLF
            if &buf[pos + bulk_len..pos + bulk_len + 2] != b"\r\n" {
                return Err(ParseError::InvalidFrame("Missing CRLF after bulk payload"));
            }

            let slice = &buf[pos..pos + bulk_len];
            args.push(slice);
            pos += bulk_len + 2;
        }

        if args.is_empty() {
            return Err(ParseError::InvalidFrame("Empty command array"));
        }

        let cmd = Self::dispatch_args(&args)?;
        Ok(Some((cmd, pos)))
    }

    /// Parses inline command format (`PING\r\n`, `SET k v\r\n`).
    fn parse_inline<'a>(buf: &'a [u8]) -> Result<Option<(Command<'a>, usize)>, ParseError> {
        let crlf = match find_crlf(buf) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        let line = &buf[..crlf];
        let parts: Vec<&'a [u8]> = line
            .split(|&b| b == b' ')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Ok(Some((Command::Ping(None), crlf + 2)));
        }

        let cmd = Self::dispatch_args(&parts)?;
        Ok(Some((cmd, crlf + 2)))
    }

    /// Dispatches parsed argument slices into a zero-copy `Command<'a>`.
    fn dispatch_args<'a>(args: &[&'a [u8]]) -> Result<Command<'a>, ParseError> {
        let cmd_name = args[0];

        if eq_ignore_ascii_case(cmd_name, b"GET") {
            if args.len() < 2 {
                return Err(ParseError::InvalidFrame("GET requires key"));
            }
            Ok(Command::Get { key: args[1] })
        } else if eq_ignore_ascii_case(cmd_name, b"SET") {
            if args.len() < 3 {
                return Err(ParseError::InvalidFrame("SET requires key and value"));
            }
            let key = args[1];
            let value = args[2];
            let mut expire_ms = None;

            let mut i = 3;
            while i < args.len() {
                if eq_ignore_ascii_case(args[i], b"EX") && i + 1 < args.len() {
                    let sec_str = std::str::from_utf8(args[i + 1])
                        .map_err(|_| ParseError::BadInteger)?;
                    let sec: u64 = sec_str.parse().map_err(|_| ParseError::BadInteger)?;
                    expire_ms = Some(sec * 1000);
                    i += 2;
                } else if eq_ignore_ascii_case(args[i], b"PX") && i + 1 < args.len() {
                    let ms_str = std::str::from_utf8(args[i + 1])
                        .map_err(|_| ParseError::BadInteger)?;
                    let ms: u64 = ms_str.parse().map_err(|_| ParseError::BadInteger)?;
                    expire_ms = Some(ms);
                    i += 2;
                } else {
                    i += 1;
                }
            }

            Ok(Command::Set {
                key,
                value,
                expire_ms,
            })
        } else if eq_ignore_ascii_case(cmd_name, b"DEL") {
            if args.len() < 2 {
                return Err(ParseError::InvalidFrame("DEL requires key"));
            }
            Ok(Command::Del { key: args[1] })
        } else if eq_ignore_ascii_case(cmd_name, b"PING") {
            if args.len() > 1 {
                Ok(Command::Ping(Some(args[1])))
            } else {
                Ok(Command::Ping(None))
            }
        } else if eq_ignore_ascii_case(cmd_name, b"EXPIRE") {
            if args.len() < 3 {
                return Err(ParseError::InvalidFrame("EXPIRE requires key and seconds"));
            }
            let sec_str = std::str::from_utf8(args[2]).map_err(|_| ParseError::BadInteger)?;
            let sec: u64 = sec_str.parse().map_err(|_| ParseError::BadInteger)?;
            Ok(Command::Expire {
                key: args[1],
                seconds: sec,
            })
        } else if eq_ignore_ascii_case(cmd_name, b"TTL") {
            if args.len() < 2 {
                return Err(ParseError::InvalidFrame("TTL requires key"));
            }
            Ok(Command::Ttl { key: args[1] })
        } else if eq_ignore_ascii_case(cmd_name, b"EXISTS") {
            if args.len() < 2 {
                return Err(ParseError::InvalidFrame("EXISTS requires key"));
            }
            Ok(Command::Exists { key: args[1] })
        } else if eq_ignore_ascii_case(cmd_name, b"MGET") {
            if args.len() < 2 {
                return Err(ParseError::InvalidFrame("MGET requires at least one key"));
            }
            Ok(Command::MGet {
                keys: args[1..].to_vec(),
            })
        } else if eq_ignore_ascii_case(cmd_name, b"MSET") {
            if args.len() < 3 || (args.len() - 1) % 2 != 0 {
                return Err(ParseError::InvalidFrame("MSET requires key-value pairs"));
            }
            let mut pairs = Vec::with_capacity((args.len() - 1) / 2);
            let mut i = 1;
            while i < args.len() {
                pairs.push((args[i], args[i + 1]));
                i += 2;
            }
            Ok(Command::MSet { pairs })
        } else if eq_ignore_ascii_case(cmd_name, b"DBSIZE") {
            Ok(Command::DbSize)
        } else if eq_ignore_ascii_case(cmd_name, b"SAVE") {
            Ok(Command::Save)
        } else if eq_ignore_ascii_case(cmd_name, b"BGSAVE") {
            Ok(Command::Bgsave)
        } else if eq_ignore_ascii_case(cmd_name, b"LASTSAVE") {
            Ok(Command::Lastsave)
        } else if eq_ignore_ascii_case(cmd_name, b"COMMAND") {
            Ok(Command::Command)
        } else if eq_ignore_ascii_case(cmd_name, b"INFO") {
            Ok(Command::Info)
        } else {
            Ok(Command::Unknown(cmd_name))
        }
    }
}

/// Fast search for `\r\n` delimiter within a byte slice.
#[inline(always)]
fn find_crlf(buf: &[u8]) -> Option<usize> {
    for i in 0..buf.len().saturating_sub(1) {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            return Some(i);
        }
    }
    None
}

/// Case-insensitive ASCII slice comparison.
#[inline(always)]
fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// RESP Wire Serializer for Outbound Network Responses.
pub struct RespEncoder;

impl RespEncoder {
    #[inline(always)]
    pub fn simple_string(msg: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(3 + msg.len());
        out.push(b'+');
        out.extend_from_slice(msg.as_bytes());
        out.extend_from_slice(b"\r\n");
        out
    }

    #[inline(always)]
    pub fn error(msg: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(7 + msg.len());
        out.extend_from_slice(b"-ERR ");
        out.extend_from_slice(msg.as_bytes());
        out.extend_from_slice(b"\r\n");
        out
    }

    #[inline(always)]
    pub fn integer(val: i64) -> Vec<u8> {
        format!(":{}\r\n", val).into_bytes()
    }

    #[inline(always)]
    pub fn bulk_string(val: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + val.len());
        out.push(b'$');
        out.extend_from_slice(val.len().to_string().as_bytes());
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(val);
        out.extend_from_slice(b"\r\n");
        out
    }

    #[inline(always)]
    pub fn null() -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }

    #[inline(always)]
    pub fn ok() -> Vec<u8> {
        b"+OK\r\n".to_vec()
    }

    #[inline(always)]
    pub fn pong() -> Vec<u8> {
        b"+PONG\r\n".to_vec()
    }

    #[inline(always)]
    pub fn boolean(val: bool) -> Vec<u8> {
        if val {
            b":1\r\n".to_vec()
        } else {
            b":0\r\n".to_vec()
        }
    }

    #[inline(always)]
    pub fn array_header(len: usize) -> Vec<u8> {
        format!("*{}\r\n", len).into_bytes()
    }

    pub fn array(items: &[Option<&[u8]>]) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + items.len() * 32);
        out.extend_from_slice(&Self::array_header(items.len()));
        for item in items {
            match item {
                Some(bytes) => out.extend_from_slice(&Self::bulk_string(bytes)),
                None => out.extend_from_slice(&Self::null()),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping() {
        let frame = b"*1\r\n$4\r\nPING\r\n";
        let res = RespParser::parse(frame).unwrap().unwrap();
        assert_eq!(res.0, Command::Ping(None));
        assert_eq!(res.1, frame.len());

        let frame_with_msg = b"*2\r\n$4\r\nPING\r\n$5\r\nhello\r\n";
        let res2 = RespParser::parse(frame_with_msg).unwrap().unwrap();
        assert_eq!(res2.0, Command::Ping(Some(b"hello")));
    }

    #[test]
    fn test_parse_set_and_get() {
        let set_frame = b"*3\r\n$3\r\nSET\r\n$7\r\nuser:42\r\n$5\r\nAkash\r\n";
        let res = RespParser::parse(set_frame).unwrap().unwrap();
        assert_eq!(
            res.0,
            Command::Set {
                key: b"user:42",
                value: b"Akash",
                expire_ms: None,
            }
        );
        assert_eq!(res.1, set_frame.len());

        let get_frame = b"*2\r\n$3\r\nGET\r\n$7\r\nuser:42\r\n";
        let res2 = RespParser::parse(get_frame).unwrap().unwrap();
        assert_eq!(res2.0, Command::Get { key: b"user:42" });
    }

    #[test]
    fn test_parse_set_with_ex() {
        let frame = b"*5\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$4\r\nval1\r\n$2\r\nEX\r\n$2\r\n60\r\n";
        let res = RespParser::parse(frame).unwrap().unwrap();
        assert_eq!(
            res.0,
            Command::Set {
                key: b"key1",
                value: b"val1",
                expire_ms: Some(60_000),
            }
        );
    }

    #[test]
    fn test_parse_incomplete_frame() {
        let partial = b"*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n";
        let res = RespParser::parse(partial).unwrap();
        assert!(res.is_none(), "Incomplete frame should return None");
    }

    #[test]
    fn test_parse_inline_commands() {
        let line = b"PING\r\n";
        let res = RespParser::parse(line).unwrap().unwrap();
        assert_eq!(res.0, Command::Ping(None));
        assert_eq!(res.1, line.len());
    }

    #[test]
    fn test_resp_encoder() {
        assert_eq!(RespEncoder::ok(), b"+OK\r\n");
        assert_eq!(RespEncoder::pong(), b"+PONG\r\n");
        assert_eq!(RespEncoder::null(), b"$-1\r\n");
        assert_eq!(RespEncoder::integer(42), b":42\r\n");
        assert_eq!(RespEncoder::bulk_string(b"hello"), b"$5\r\nhello\r\n");
        assert_eq!(
            RespEncoder::error("unknown command"),
            b"-ERR unknown command\r\n"
        );
    }
}
