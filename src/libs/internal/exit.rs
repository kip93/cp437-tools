use std::{
    fmt::{self, Display, Formatter},
    io,
    num::ParseIntError,
    num::TryFromIntError,
    ops::{ControlFlow, FromResidual, Try},
    process::{ExitCode as StdExitCode, Termination},
    ptr,
    string::FromUtf8Error,
};

#[cfg(feature = "binaries")]
use png::EncodingError;

use crate::internal::help;

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExitCode {
    OK = 0x00,
    FAIL(String) = 0x01,
    USAGE(String) = 0x7E,
    ERROR(String) = 0x7F,
}

impl ExitCode {
    #[inline]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        return self == &ExitCode::OK;
    }

    #[inline]
    #[must_use]
    pub fn is_err(&self) -> bool {
        return !self.is_ok();
    }

    pub fn print(&self) {
        if self != &ExitCode::OK {
            if self.as_str() != "" {
                eprintln!("\x1B[31mERROR: {self}\x1B[0m");
            }
            if let ExitCode::USAGE(_) = self {
                help::print("cp437-tools").expect("Valid command");
            }
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        return match self {
            ExitCode::OK => "",
            ExitCode::USAGE(s) | ExitCode::FAIL(s) | ExitCode::ERROR(s) => s.as_str(),
        };
    }

    #[must_use]
    pub fn as_string(&self) -> String {
        return match self {
            ExitCode::OK => String::default(),
            ExitCode::USAGE(s) | ExitCode::FAIL(s) | ExitCode::ERROR(s) => s.clone(),
        };
    }

    #[must_use]
    pub fn as_u8(&self) -> u8 {
        // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        return unsafe { *ptr::from_ref::<Self>(self).cast::<u8>() };
    }
}

impl Display for ExitCode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.as_string().fmt(f);
    }
}

impl From<ExitCode> for u8 {
    #[inline]
    fn from(code: ExitCode) -> u8 {
        return code.as_u8();
    }
}

impl<'a> From<&'a ExitCode> for &'a str {
    #[inline]
    fn from(code: &'a ExitCode) -> &'a str {
        return code.as_str();
    }
}

impl From<ExitCode> for String {
    #[inline]
    fn from(code: ExitCode) -> String {
        return code.to_string();
    }
}

impl From<()> for ExitCode {
    #[inline]
    fn from(_: ()) -> ExitCode {
        return ExitCode::OK;
    }
}

impl From<String> for ExitCode {
    #[inline]
    fn from(msg: String) -> ExitCode {
        return ExitCode::FAIL(msg);
    }
}

impl<'a> From<&'a str> for ExitCode {
    #[inline]
    fn from(msg: &'a str) -> ExitCode {
        return ExitCode::from(String::from(msg));
    }
}

impl From<FromUtf8Error> for ExitCode {
    #[inline]
    fn from(err: FromUtf8Error) -> ExitCode {
        return ExitCode::ERROR(err.to_string());
    }
}

impl From<TryFromIntError> for ExitCode {
    #[inline]
    fn from(err: TryFromIntError) -> ExitCode {
        return ExitCode::ERROR(err.to_string());
    }
}

impl From<io::Error> for ExitCode {
    #[inline]
    fn from(err: io::Error) -> ExitCode {
        return ExitCode::ERROR(err.to_string());
    }
}

impl From<ParseIntError> for ExitCode {
    #[inline]
    fn from(err: ParseIntError) -> ExitCode {
        return ExitCode::ERROR(err.to_string());
    }
}

#[cfg(feature = "binaries")]
impl From<EncodingError> for ExitCode {
    #[inline]
    fn from(err: EncodingError) -> ExitCode {
        return ExitCode::ERROR(err.to_string());
    }
}

impl<T, E> From<Result<T, E>> for ExitCode
where
    ExitCode: From<E>,
{
    #[inline]
    fn from(result: Result<T, E>) -> ExitCode {
        return match result {
            Ok(_) => ExitCode::OK,
            Err(err) => ExitCode::from(err),
        };
    }
}

impl<E> From<ExitCode> for Result<(), E>
where
    E: From<ExitCode>,
{
    #[inline]
    fn from(exit_code: ExitCode) -> Result<(), E> {
        return match exit_code {
            ExitCode::OK => Ok(()),
            _ => Err(E::from(exit_code)),
        };
    }
}

// TODO https://github.com/rust-lang/rust/issues/84277

impl<T> FromResidual<T> for ExitCode
where
    ExitCode: From<T>,
{
    #[inline]
    fn from_residual(residual: T) -> Self {
        let code = ExitCode::from(residual);
        assert_ne!(code, ExitCode::OK, "ExitCode::OK is not a residual");
        return code;
    }
}

impl Try for ExitCode {
    type Output = Self;
    type Residual = Self;

    fn from_output(output: Self) -> Self {
        assert_eq!(output, ExitCode::OK, "Output must be ExitCode::OK");
        return output;
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self, Self> {
        return match self {
            ExitCode::OK => ControlFlow::Continue(self),
            _ => ControlFlow::Break(self),
        };
    }
}

impl Termination for ExitCode {
    #[inline]
    fn report(self) -> StdExitCode {
        return StdExitCode::from(u8::from(self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    const MSG: &str = "foo";

    #[test]
    fn is_ok() {
        assert!(ok().is_ok());
    }

    #[test]
    fn is_err() {
        assert!(err().is_err());
    }

    #[test]
    fn ok_code() {
        assert_eq!(u8::from(ok()), 0x00);
    }

    #[test]
    fn error_code() {
        assert_eq!(u8::from(err()), 0x7F);
    }

    #[test]
    fn ok_message() {
        assert_eq!(String::from(ok()), "");
    }

    #[test]
    fn error_message() {
        assert_eq!(String::from(err()), MSG);
    }

    #[test]
    fn display_message() {
        assert_eq!(format!("{}", err()), MSG);
    }

    #[test]
    fn try_ok() {
        assert_eq!(wrap(ok()), ok());
    }

    #[test]
    fn try_error() {
        assert_eq!(wrap(err()), err());
    }

    #[test]
    fn from_ok() {
        assert_eq!(ExitCode::from(ok()), ok());
    }

    #[test]
    fn from_error() {
        assert_eq!(ExitCode::from(err()), err());
    }

    #[test]
    fn from_tuple() {
        assert_eq!(ExitCode::from(()), ok());
    }

    #[test]
    fn from_str() {
        assert_eq!(ExitCode::from(MSG), fail());
    }

    #[test]
    fn from_string() {
        assert_eq!(ExitCode::from(String::from(MSG)), fail());
    }

    #[test]
    fn from_io_error() {
        assert_eq!(ExitCode::from(io_err()), err());
    }

    #[test]
    fn from_residual_error() {
        assert_eq!(ExitCode::from_residual(err()), err());
    }

    #[test]
    fn from_residual_string() {
        assert_eq!(ExitCode::from_residual(String::from(MSG)), fail());
    }

    #[test]
    fn from_residual_str() {
        assert_eq!(ExitCode::from_residual(MSG), fail());
    }

    #[test]
    fn from_residual_io_error() {
        assert_eq!(ExitCode::from_residual(io_err()), err());
    }

    #[test]
    fn from_residual_result_error() {
        assert_eq!(ExitCode::from_residual(Err::<(), ExitCode>(err())), err());
    }

    #[test]
    fn from_residual_result_string() {
        assert_eq!(ExitCode::from_residual(Err::<(), String>(String::from(MSG))), fail());
    }

    #[test]
    fn from_residual_result_str() {
        assert_eq!(ExitCode::from_residual(Err::<(), &str>(MSG)), fail());
    }

    #[test]
    fn from_residual_result_io_error() {
        assert_eq!(ExitCode::from_residual(Err::<(), io::Error>(io_err())), err());
    }

    #[test]
    #[should_panic = "ExitCode::OK is not a residual"]
    fn from_residual_ok() {
        ExitCode::from_residual(ok());
    }

    #[test]
    fn from_output_ok() {
        assert_eq!(ExitCode::from_output(ok()), ok());
    }

    #[test]
    #[should_panic = "Output must be ExitCode::OK"]
    fn from_output_err() {
        ExitCode::from_output(err());
    }

    #[inline]
    fn wrap(exit_code: ExitCode) -> ExitCode {
        exit_code?;
        return ExitCode::OK;
    }

    fn ok() -> ExitCode {
        return ExitCode::OK;
    }

    fn err() -> ExitCode {
        return ExitCode::ERROR(String::from(MSG));
    }

    fn fail() -> ExitCode {
        return ExitCode::FAIL(String::from(MSG));
    }

    fn io_err() -> io::Error {
        return io::Error::new(io::ErrorKind::Other, MSG);
    }
}
