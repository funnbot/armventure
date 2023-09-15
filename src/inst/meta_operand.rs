macro_rules! _arg_type_impl {
    ($name:ident ()) => {
        $crate::inst::operand::op::$name
    };
}

macro_rules! _arg_encode_impl {
    (Sf()) => {
        enc::Sf
    };
    (Gpr()) => {
        enc::Gpr
    };
    (Gpr(AllowSp)) => {
        enc::GprOrSp
    };
    (Gpr(AllowZr)) => {
        enc::GprOrZr
    };
    (UImm($bits:literal, Align = $align:literal)) => {
        enc::UImmAlign<$bits, $align>
    };
    (SImm($bits:literal, Align = $align:literal)) => {
        enc::SImmAlign<$bits, $align>
    };
    (UImm($bits:literal)) => {
        enc::UImm<$bits>
    };
    (SImm($bits:literal)) => {
        enc::SImm<$bits>
    };
    (Shift(Kind)) => {
        enc::ShiftKind
    };
    (Shift(Amount($bits:literal))) => {
        enc::ShiftAmount<$bits>
    };
    (ShiftConst($kind:ident, $amt:literal)) => {
        enc::ShiftConst<{$crate::inst::operand::ShiftKind::$kind as u8}, $amt>
    };
    (Extend(Shift)) => {
        enc::ExtendLShift
    };
    (Cond()) => {
        enc::Cond
    };
    (Label($name:ident $opts:tt)) => {
        enc::Label<
            $crate::inst::meta_operand::_arg_encode_impl!($name $opts),
        >
    };
    ($ident:ident $opts:tt) => {
        compile_error!(concat!("unknown encode argument: ", stringify!($ident), stringify!($opts)));
    };
}

macro_rules! _arg_encode {
    (B($binary:literal) $s:tt $e:tt) => {
        Ok($e.push(IntN(
            $binary,
            $crate::inst::util::binary_string(stringify!($binary)),
        )))
    };
    (Extend(Kind) $s:tt $e:tt $i:tt) => {
        (|| {
            enc::ExtendKind::valid_width(&$s.$i, &$s.2)?;
            let value = enc::ExtendKind::encode(&$s.$i, $e)?;
            $e.push_n(value);
            Ok(())
        })()
    };
    ($ident:ident $opts:tt $s:tt $e:tt $( $i:tt )?) => {
        <$crate::inst::meta_operand::_arg_encode_impl!($ident $opts)>
            ::encode(&$s.$($i)?, $e).map(|v| $e.push_n(v))
    }
}

macro_rules! _arg_parse {
    (Opt $iter:ident) => {
        $crate::inst::util::parse_arg_opt($iter.next())
    };
    ($name:ident $iter:ident) => {
        $crate::inst::util::parse_arg($iter.next())
    };
}

macro_rules! _arg_kind {
    (Opt($name:ident $opts:tt)) => {
        $crate::inst::util::Param::Opt($crate::inst::operand::Kind::$name)
    };
    ($name:ident $opts:tt) => {
        $crate::inst::util::Param::Req($crate::inst::operand::Kind::$name)
    };
}

macro_rules! _arg_type {
    (Opt($name:ident $opts:tt)) => {
        ::std::option::Option<$crate::inst::meta_operand::_arg_type_impl!($name $opts)>
    };
    ($name:ident $opts:tt) => {
        _arg_type_impl!($name $opts)
    }
}

pub(super) use _arg_encode;
pub(super) use _arg_encode_impl;
pub(super) use _arg_kind;
pub(super) use _arg_parse;
pub(super) use _arg_type;
pub(super) use _arg_type_impl;
