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
    /// 110 - Restart marker reply.
    (110, RESTART_MARKER_REPLY);
    /// 120 - Service ready in nnn minutes.
    (120, SERVICE_READY);
    /// 125 - Data connection already open; transfer starting.
    (125, DATA_CONNECTION_ALREADY_OPEN);
    /// 150 - File status okay; about to open data connection.
    (150, OPENING_DATA_CONNECTION);

    /// 200 - Command okay
    (200, COMMAND_OKAY);
    /// 211 - System status, or system help reply.
    (211, SYSTEM_STATUS);
    /// 212 - Directory status.
    (212, DIRECTORY_STATUS);
    /// 213 - File status.
    (213, FILE_STATUS);
    /// 214 - Help message.
    (214, HELP);
    /// 215 - NAME system type.
    /// Where NAME is an official system name from the list in the Assigned Numbers document.
    (215, SYSTEM_TYPE);
    /// 220 - Service ready for new user.
    (220, READY);
    /// 221 - Service closing control connection. Logged out if appropriate.
    (221, CLOSING_CONTROL_CONNECTION);
    /// Data connection open; no transfer in progress.
    (225, DATA_CONNECTION_OPEN);
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
    /// 332 - Need account for login.
    (332, NEED_ACCOUNT_FOR_LOGIN);
    /// 350 - Requested file action pending further information.
    (350, REQUESTED_FILE_ACTION_PENDING_FURTHER_INFORMATION);

    /// 421 - Service not available, closing control connection.
    /// This may be a reply to any command if the service knows it must shut down.
    (421, SERVICE_NOT_AVAILABLE);
    /// 425 - Can't open data connection.
    (425, CANT_OPEN_DATA_CONNECTION);
    /// 426 - Connection closed; transfer aborted.
    (426, CONNECTION_CLOSED);
    /// 450 - Requested file action not taken.
    /// File unavailable (e.g., file busy).
    (450, FILE_ACTION_UNAVAILABLE);
    /// 451 - Requested action aborted: local error in processing.
    (451, LOCAL_ERROR_IN_PROCESSING);
    /// 452 - Requested action not taken.
    /// Insufficient storage space in system.
    (452, INSUFFICIENT_STORAGE_SPACE);

    /// 500 - Syntax error, command unrecognized.
    /// This may include errors such as command line too long.
    (500, UNRECOGNIZED_COMMAND);
    /// 501 - Syntax error in parameters or arguments.
    (501, UNRECOGNIZED_ARGUMENTS);
    /// 502 - Command not implemented.
    (502, NOT_IMPLEMENTED);
    /// 503 - Bad sequence of commands.
    (503, BAD_SEQUENCE);
    /// 504 - Command not implemented for that parameter.
    (504, NOT_IMPLEMENTED_FOR_PARAMETER);
    /// 530 - Not logged in.
    (530, NOT_LOGGED_IN);
    /// 532 - Need account for storing files.
    (532, NEED_ACCOUNT_FOR_STORING_FILES);
    /// 550 - Requested action not taken.
    /// File unavailable (e.g., file not found, no access).
    (550, FILE_UNAVAILABLE);
    /// 551 - Requested action aborted: page type unknown.
    (551, PAGE_TYPE_UNKNOWN);
    /// 552 - Requested file action aborted.
    /// Exceeded storage allocation (for current directory or dataset).
    (552, EXCEEDED_STORAGE_ALLOCATION);
    /// 553 - Requested action not taken.
    /// File name not allowed.
    (553, FILE_NAME_NOT_ALLOWED);
}
