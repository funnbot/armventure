use super::meta::*;

pub enum Variant {
    Default,
    ShiftedRegister,
    ExtendedRegister,
    Immediate,
    Condition,
}

def_instrs! {
    UDF () (B(0b10));
    ADD Immediate
        (Gpr() Gpr() Imm() Opt(Shift()))
        (Sf():0 B(0b00100010) ShiftConst(LSL, 12):3 Gpr(AllowSp):1 UImm(12):2 Gpr(AllowSp):0),
        ShiftedRegister
        // docs say Shift is optional, but that conflicts with ExtendedRegister,
        // and need to favor that since it allows special registers in the common `ADD Gpr, Gpr, Gpr`
        (Gpr() Gpr() Gpr() Shift())
        (Sf():0 B(0b0001011) Shift(Kind):3 B(0b0) Gpr():2 Shift(Amount(6)):3 Gpr():1 Gpr():0),
        ExtendedRegister
        (Gpr() Gpr() Gpr() Opt(Extend()))
        (Sf():2 B(0b0001011001) Gpr(AllowZr):2 Extend(Kind):3 Extend(Shift):3 Gpr(AllowSp):1 Gpr(AllowSp):0);

    ADDS ExtendedRegister
        (Gpr() Gpr() Gpr() Opt(Extend()))
        (Sf():0 B(0b0101011001) Gpr():2 Extend(Kind):3 Extend(Shift):3 Gpr():1 Gpr():0);

    B   Default
        (Label())
        (B(0b000101) Label(SImm(26, Align = 2)):0),
        Condition
        (Cond() Label())
        (B(0b01010100) Label(SImm(19, Align = 2)):1 B(0b0) Cond():0);
}

trait Instr {
    fn gpr_0(&self) -> u8;
    fn gpr_1(&self) -> u8;
}
