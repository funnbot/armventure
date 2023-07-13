use std::convert::Infallible;
use std::fmt;
use std::ops::FromResidual;
use std::process::{ExitCode, Termination};

pub struct Probably;

impl std::ops::FromResidual<Option<!>> for Probably {
    #[inline]
    #[track_caller]
    fn from_residual(_: Option<!>) -> Self {
        panic!("try value is None");
    }
}

impl<E: fmt::Debug> FromResidual<Result<!, E>> for Probably {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<!, E>) -> Self {
        unsafe { panic!("try value is: {:?}", residual.unwrap_err_unchecked()) }
    }
}

impl std::ops::FromResidual<Option<Infallible>> for Probably {
    #[inline]
    #[track_caller]
    fn from_residual(_: Option<Infallible>) -> Self {
        panic!("try value is: None");
    }
}

impl<E: fmt::Debug> FromResidual<Result<Infallible, E>> for Probably {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        unsafe { panic!("try value is: {:?}", residual.unwrap_err_unchecked()) }
    }
}

impl Termination for Probably {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}
