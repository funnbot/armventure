use std::ops::{ControlFlow, Try};

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
macro_rules! simpl {
    { Default $( [ $( $targs:tt )* ] )? |$Ty:ty| $body:expr } => {
        impl$( <$( $targs )*> )? ::std::default::Default for $Ty {
            fn default() -> Self {
                $body
            }
        }
    };
    { TryFrom $( [ $( $targs:tt )* ] )? |$value:ident : $From:ty => ($To:ty, $Error:ty)| $body:expr } => {
        impl$( <$( $targs )*> )? ::std::convert::TryFrom<$From> for $To {
            type Error = $Error;
            fn try_from($value : $From) -> ::std::result::Result<Self, Self::Error> {
                $body
            }
        }
    };
    { From $( [ $( $targs:tt )* ] )? |$value:ident : $From:ty => $To:ty| $body:expr } => {
        impl$( <$( $targs )*> )? ::std::convert::From<$From> for $To {
            fn from($value : $From) -> Self {
                $body
            }
        }
    };


    { Debug $( [ $( $targs:tt )* ] )? |$self:ident : $Ty:ty| $( $body:tt )+ } => {
        impl$( <$( $targs )*> )? ::std::fmt::Debug for $Ty {
            fn fmt(& $self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Formatter::write_fmt(f, ::std::format_args!( $( $body )+ ))
            }
        }
    };
    { Debug $( [ $( $targs:tt )* ] )? |$self:ident : $Ty:ty, $f:ident| $body:expr } => {
        impl$( <$( $targs )*> )? ::std::fmt::Debug for $Ty {
            fn fmt(& $self, $f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                $body
            }
        }
    };

    { Display $( [ $( $targs:tt )* ] )? |$self:ident : $Ty:ty| $( $body:tt )+ } => {
        impl$( <$( $targs )*> )? ::std::fmt::Display for $Ty {
            fn fmt(& $self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Formatter::write_fmt(f, ::std::format_args!( $( $body )+ ))
            }
        }
    };
    { Display $( [ $( $targs:tt )* ] )? |$self:ident : $Ty:ty, $f:ident| $body:expr } => {
        impl$( <$( $targs )*> )? ::std::fmt::Display for $Ty {
            fn fmt(& $self, $f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                $body
            }
        }
    };
    { MaybeDisplay $( [ $( $targs:tt )* ] )? |$self:ident : $Ty:ty| $body:expr } => {
        impl$( <$( $targs )*> )? $crate::inst::util::MaybeDisplay<$Ty> for $Ty {
            fn maybe_display(& $self) -> $crate::inst::util::OptionDisplay<Self> {
                use $crate::inst::util::OptionDisplay::*;
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! simpls {
    { $( $tt:tt )* } => {
        $( simpl! $tt )*
    };
}

/// writes a `sep` separated list of `MaybeDisplay<T>` with `fmt` to the write! macro
/// if the value is None, it is not written and it will not repeat the separator
#[macro_export]
macro_rules! write_maybe_join {
    ($f:ident, $fmt:literal, $sep:literal $(,)?) => {
        Ok(())
    };
    ($f:ident, $fmt:literal, $sep:literal, $($values:expr),* $(,)?) => {
        'res: {
            use $crate::inst::util::MaybeDisplay as _;
            use $crate::inst::util::OptionDisplay::{DisplayNone, DisplaySome};
            let mut needs_sep = false;
            $(
                match $values.maybe_display() {
                    DisplaySome(val) => {
                        let result = if needs_sep {
                            write!($f, concat!($sep, $fmt), val)
                        } else {
                            write!($f, $fmt, val)
                        };
                        #[allow(unused_assignments)]
                        if result.is_ok() {
                            needs_sep = true;
                        } else {
                            break 'res result;
                        }
                    }
                    DisplayNone => (),
                }
            )+
            Ok(())
        }
    }
}

/// writes a `sep` separated list of values with `fmt` to the write! macro
#[macro_export]
macro_rules! write_join {
    ($f:ident, $fmt:literal, $sep:literal $(,)?) => {
        Ok(())
    };
    ($f:ident, $fmt:literal, $sep:literal, $value:expr $(,)?) => {
        write!($f, $fmt, $value)
    };
    ($f:ident, $fmt:literal, $sep:literal, $value0:expr, $($values:expr),* $(,)?) => {
        write!($f,
            concat!(
                $fmt, $(
                    ${ignore(values)}
                    $sep, $fmt
                ),*
            ),
            $value0, $($values),*
        )
    }
}

// #[macro_export]
// macro_rules! tmod {
//     {
//         $({$(
//             $item:stmt;
//         )*})?
//         $(
//             test $test_name:ident $( -> $test_ret:ty )? $test_body:block
//         )*
//     } => {
//         #[cfg(test)]
//         mod tests {
//             use super::*;
//                 $($( $item )*)?
//                 $(
//                     #[test]
//                     fn $test_name() $( -> $test_ret )? $test_body
//                 )*
//         }
//     };
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn unwrap_macro() {}
// }
