use std::ops::ControlFlow;
use std::ops::Try;

#[macro_export]
macro_rules! unwrap {
    ( $value:expr; else $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue(c) => c,
            ::std::ops::ControlFlow::Break(..) => $break_expr,
        }
    };
    ( $value:expr; else $break_pat:pat => $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue(c) => c,
            ::std::ops::ControlFlow::Break($break_pat) => $break_expr,
            #[allow(unreachable_patterns)]
            ::std::ops::ControlFlow::Break(..) => {
                unreachable!("break is infallible")
            }
        }
    };

    ( $value:expr; $cont_expr:expr, else $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue(..) => $cont_expr,
            ::std::ops::ControlFlow::Break(..) => $break_expr,
        }
    };
    ( $value:expr; $cont_expr:expr, else $pat:pat => $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue(..) => $cont_expr,
            ::std::ops::ControlFlow::Break($pat) => $break_expr,
            #[allow(unreachable_patterns)]
            ::std::ops::ControlFlow::Break(..) => {
                unreachable!("break is infallible")
            }
        }
    };

    ( $value:expr; $cont_pat:pat => $cont_expr:expr, else $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue($cont_pat) => $cont_expr,
            ::std::ops::ControlFlow::Break(..) => $break_expr,
        }
    };
    ( $value:expr; $cont_pat:pat => $cont_expr:expr, else $break_pat:pat => $break_expr:expr ) => {
        match $value.branch() {
            ::std::ops::ControlFlow::Continue($cont_pat) => $cont_expr,
            ::std::ops::ControlFlow::Break($break_pat) => $break_expr,
            #[allow(unreachable_patterns)]
            ::std::ops::ControlFlow::Break(..) => {
                unreachable!("break is infallible")
            }
        }
    };
}

#[macro_export]
macro_rules! batch_assert_matches {
    { $( $value:expr => $pattern:pat),+ /* optional trailing , */ $(,)? } =>
    {
        $(
            match $value {
                $pattern => { /* do nothing */},
                ref left_value => std::panic!("assertion failed: pattern not matched, {}:{}:{}\nPattern: {}\nValue: {:?}",
                     std::file!(),
                     std::line!() + ${index()} + 1, // assuming called with each match on newline
                     std::column!(), // column of invocation, use this or 1?
                     stringify!($pattern),
                     left_value),
            }
        );+ // replace repetition , with ;
    }
}

#[macro_export]
macro_rules! impl_display {
    (|$this:ident : $Ty:ty| $fmt:literal $($rest:tt)*) => {
        impl ::std::fmt::Display for $Ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $this = self;
                ::std::fmt::Formatter::write_fmt(f, ::std::format_args!($fmt $($rest)*))
            }
        }
    };
    (|$this:ident : $Ty:ty, $f:ident| $($tt:tt)+) => {
        impl ::std::fmt::Display for $Ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $this = self;
                let $f = f;
                {
                    $($tt)+
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_debug {
    (|$this:ident : $Ty:ty| $fmt:literal $($rest:tt)*) => {
        impl ::std::fmt::Debug for $Ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $this = self;
                ::std::fmt::Formatter::write_fmt(f, ::std::format_args!($fmt $($rest)*))
            }
        }
    };
    (|$this:ident : $Ty:ty, $f:ident| $($tt:tt)+) => {
        impl ::std::fmt::Debug for $Ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $this = self;
                let $f = f;
                {
                    $($tt)+
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_macro() {}
}
