use super::meta::*;

pub enum Variant {
    Default,
    ShiftedRegister,
    ExtendedRegister,
    Immediate,
    Condition,
}

def_instrs! {
    B Default (Label(SImm(26, B4))) (B(0b000101) Arg():0);
    //ADD Immediate
    //    (Gpr(AllowSp) Gpr(AllowSp) UImm(12) Opt(ShiftConst(LSL, 12)))
    //    (Sf(Gpr0):0 B(0b00100010) Arg():3 Arg():2 Arg():1 Arg():0);
    //     ShiftedRegister
    //     (Gpr() Gpr() Gpr() Opt(Shift(6)))
    //     (Sf(Gpr0) Const(0b0001011) Shift(Kind(3)) Const(0b0) Gpr(2) Shift(Amount(3)) Gpr(1) Gpr(0)),
    //     ExtendedRegister
    //     (Gpr(AllowSp) Gpr(AllowSp) Gpr(AllowZr) Opt(Extend()))
    //     (Sf(Gpr0) Const(0b0001011001) Gpr(2) Extend(Kind(3)) Extend(Shift(3)) Gpr(1) Gpr(0));

    //B   Default (Label(SImm(26, 2))) (B(0b000101) Label(Arg()):0),
    //    Condition (Cond() Label(SImm(19, 2))) (B(0b01010100) Label(Arg()):1);
}
