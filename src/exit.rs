use std::{
    fmt::{self, Display, Formatter},
    process::{ExitCode as StdExitCode, Termination},
};

#[doc(hidden)] // Internal impl detail
#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExitCode {
    OK = 0x00,
    USAGE(String) = 0x01,
    FAIL(String) = 0x02,
    ERROR(String) = 0x7F,
}

impl ExitCode {
    #[inline]
    fn as_str(&self) -> String {
        return match self {
            ExitCode::OK => String::default(),
            ExitCode::USAGE(s) | ExitCode::FAIL(s) | ExitCode::ERROR(s) => s.clone(),
        };
    }

    #[inline]
    fn as_u8(&self) -> u8 {
        // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        return unsafe { *(self as *const Self as *const u8) };
    }
}

impl Display for ExitCode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.as_str().fmt(f);
    }
}

impl From<ExitCode> for u8 {
    #[inline]
    fn from(code: ExitCode) -> u8 {
        return code.as_u8();
    }
}

impl From<ExitCode> for String {
    #[inline]
    fn from(code: ExitCode) -> String {
        return code.as_str();
    }
}

impl Termination for ExitCode {
    #[inline]
    fn report(self) -> StdExitCode {
        return StdExitCode::from(u8::from(self));
    }
}
