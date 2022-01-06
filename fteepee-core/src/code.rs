use core::fmt;

macro_rules! impl_codes {
	(
        $(
            $(#[$docs:meta])*
            ($code:expr, $name:ident);
        )+
    ) => {
		impl Code {
			$(
				$(#[$docs])*
					pub const $name: Code = Code(unsafe { core::num::NonZeroU16::new_unchecked($code)});
			)*
		}

		impl core::convert::TryFrom<[u8; 3]> for Code {
			type Error = crate::Error;

			fn try_from(bytes: [u8; 3]) -> $crate::Result<Self> {
				let b = &bytes[..];
				$(
					if stringify!($code).as_bytes() == b {
						return Ok(Code::$name);
					}
				)*
				Err(crate::Error::InvalidCode(bytes))
			}
		}
	};
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Code(core::num::NonZeroU16);

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl_codes! {
    /// 125 - Data connection already open; transfer starting.
    (125, DATA_CONNECTION_ALREADY_OPEN);
    /// 150 - File status okay; about to open data connection.
    (150, OPENING_DATA_CONNECTION);

    /// 211 - System status, or system help reply.
    (211, STATUS);
    /// 220 - Service ready for new user.
    (220, READY);
    /// 221 - Service closing control connection. Logged out if appropriate.
    (221, CLOSING_CONTROL_CONNECTION);
    /// 226 - Closing data connection. Requested file action successful (for
    /// example, file transfer or file abort).
    (226, CLOSING_DATA_CONNECTION);
    /// 227 - Entering Passive Mode (h1,h2,h3,h4,p1,p2).
    (227, ENTERING_PASSIVE_MODE);
    /// 230 - User logged in, proceed.
    (230, LOGGED_IN);
    /// 250 - Requested file action okay, completed.
    (250, REQUESTED_FILE_ACTION_OKAY);
    /// 257 - "PATHNAME" created.
    (257, CREATED);

    /// 330 - User name okay, need password.
    (331, PASSWORD_REQUIRED);

    /// 500 - Syntax error, command unrecognized. This may include errors such
    /// as command line too long.
    (500, UNRECOGNIZED_COMMAND);
    /// 501 - Syntax error in parameters or arguments.
    (501, UNRECOGNIZED_ARGUMENTS);
    /// 502 - Command not implemented.
    (502, NOT_IMPLEMENTED);
}
