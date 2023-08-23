macro_rules! _arg_type_impl {
    (Gpr()) => ($crate::inst::operand::Gpr);
    (Gpr(AllowSp)) => ($crate::inst::operand::GprOrSp);
    (Gpr(AllowZr)) => ($crate::inst::operand::GprOrZr);
    (UImm( $bits:literal $( , $align:literal )? )) =>
        ($crate::inst::operand::UImm<$bits $(, $align)?>);
    (SImm( $bits:literal $( , $align:ident )? )) =>
        ($crate::inst::operand::SImm<$bits $(, {$crate::inst::operand::AlignBits::$align as u8})?>);
    (ShiftConst($kind:ident, $amt:literal)) =>
        ($crate::inst::operand::ShiftConst<{$crate::inst::operand::ShiftKind::$kind as u8}, $amt>);
    (Shift($bits:literal)) => ($crate::inst::operand::Shift<$bits>);
    (Extend()) => ($crate::inst::operand::Extend);
    // (Label(UImm($bits:literal $( , $align:literal )?))) =>
    //     ($crate::inst::operand::LabelUImm<$bits $(, $align)?>);
    // (Label(SImm($bits:literal $( , $align:literal )?))) =>
    //     ($crate::inst::operand::LabelSImm<$bits $(, $align)?>);
    (Label($kind:ident $opts:tt)) => {
        $crate::inst::operand::op::Label<
            $crate::inst::meta_operand::_arg_type_impl!($kind $opts) >
    };
    (Cond()) => ($crate::inst::operand::Cond);
    ($ident:ident $opts:tt) => {
        compile_error!("unknown argument");
    };
}

macro_rules! _arg_encode_impl {
    (B($binary:literal) $s:tt) => {
        IntN(
            $binary,
            $crate::inst::util::binary_string(stringify!($binary)),
        )
    };
    (Sf(Gpr0) $s:tt) => {
        IntN(($s.size == $crate::inst::operand::GprSize::B8) as u32, 1)
    };
    (Arg()) => {
        $crate::inst::operand::enc::Op
    };
    (Arg(Kind)) => {
        $crate::inst::operand::enc::OpKind
    };
    (Gpr() $s:tt) => {
        $s
    };
    (UImm($idx:tt) $s:tt) => {
        $s
    };
    (SImm() $s:tt) => {
        $s
    };
    (Shift(Kind($idx:tt)) $s:tt) => {
        $s.kind
    };
    (Shift(Amount($idx:tt)) $s:tt) => {
        $s.amount
    };
    (ShiftConst($idx:tt) $s:tt) => {
        $s
    };
    (Extend(Kind($idx:tt)) $s:tt) => {
        $s.kind
    };
    (Extend(Shift($idx:tt)) $s:tt) => {
        $s.lsl_amount
    };
    (Cond()) => {
        $s
    };
    ($ident:ident $opts:tt) => {
        compile_error!("unknown encode argument");
    };
}

macro_rules! _arg_encode {
    (PcRel($name:ident $opts:tt) $s:tt $e:tt) => {

    };
    (B($binary:literal) $s:tt $e:tt) => {
        $e.push(IntN(
            $binary,
            $crate::inst::util::binary_string(stringify!($binary)),
        ))
    };
    ($ident:ident $opts:tt $s:tt $e:tt) => {
        {
            let value = <$crate::inst::meta_operand::_arg_encode_impl!($ident $opts)>::encode($s, $e);
            $e.push(value)
        }
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
        $crate::inst::util::Param::Opt(
            <_arg_type_impl!($name $opts) as
                $crate::inst::operand::Operand>::KIND)
    };
    ($name:ident $opts:tt) => {
        $crate::inst::util::Param::Req(
            <_arg_type_impl!($name $opts) as
                $crate::inst::operand::Operand>::KIND)
    }
}

macro_rules! _arg_type {
    (Opt($name:ident $opts:tt)) => {
        _arg_type_impl!($name $opts)
    };
    ($name:ident $opts:tt) => {
        _arg_type_impl!($name $opts)
    }
}

pub(super) use {_arg_encode, _arg_encode_impl, _arg_kind, _arg_parse, _arg_type, _arg_type_impl};
