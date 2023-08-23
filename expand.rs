mod def {
    use super::meta::*;
    pub enum Variant {
        Default,
        ShiftedRegister,
        ExtendedRegister,
        Immediate,
        Condition,
    }
    pub enum Mnemonic {
        ADD,
        B,
    }
    impl std::str::FromStr for Mnemonic {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let hash = crate::inst::util::str_hash(s);
            if crate::inst::util::str_hash("ADD") == hash {
                return Ok(Mnemonic::ADD);
            }
            if crate::inst::util::str_hash("B") == hash {
                return Ok(Mnemonic::B);
            }
            Err(())
        }
    }
    pub mod parse {
        use crate::inst::meta::*;
        use super::Variant;
        pub trait Instruction
        where
            Self: Sized,
        {
            const MNEM: super::Mnemonic;
            const VARIANT: Variant = Variant::Default;
            const ARGS: &'static [crate::inst::util::Param];
            fn from_args<'a, A, I: Iterator>(
                iter: I,
            ) -> ::std::result::Result<Self, crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: crate::inst::Arg,
                A: 'a;
            fn encode<
                E: crate::inst::Emitter,
                RL: crate::inst::Resolver<::lasso::Spur, u64>,
            >(&self, e: &mut E, rl: &mut RL);
        }
        #[allow(non_camel_case_types)]
        pub struct ADD_Immediate(
            crate::inst::operand::GprOrSp,
            crate::inst::operand::GprOrSp,
            crate::inst::operand::UImm<12>,
            crate::inst::operand::ShiftConst<
                { crate::inst::operand::ShiftKind::LSL as u8 },
                12,
            >,
        );
        impl From<ADD_Immediate> for InstrSet {
            fn from(value: ADD_Immediate) -> Self {
                InstrSet::ADD_Immediate(value)
            }
        }
        impl Instruction for ADD_Immediate {
            const MNEM: super::Mnemonic = super::Mnemonic::ADD;
            const VARIANT: Variant = Variant::Immediate;
            const ARGS: &'static [crate::inst::util::Param] = &[
                crate::inst::util::Param::Req(
                    <crate::inst::operand::GprOrSp as crate::inst::operand::Operand>::KIND,
                ),
                crate::inst::util::Param::Req(
                    <crate::inst::operand::GprOrSp as crate::inst::operand::Operand>::KIND,
                ),
                crate::inst::util::Param::Req(
                    <crate::inst::operand::UImm<
                        12,
                    > as crate::inst::operand::Operand>::KIND,
                ),
                crate::inst::util::Param::Opt(
                    <crate::inst::operand::ShiftConst<
                        { crate::inst::operand::ShiftKind::LSL as u8 },
                        12,
                    > as crate::inst::operand::Operand>::KIND,
                ),
            ];
            #[allow(unused_mut, unused_variables)]
            fn from_args<'a, A, I: Iterator>(
                mut iter: I,
            ) -> ::std::result::Result<Self, crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: crate::inst::Arg,
                A: 'a,
            {
                Ok(
                    Self(
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 0))?,
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 1))?,
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 2))?,
                        match iter.next() {
                            Some(value) => {
                                crate::inst::Arg::parse(value)
                                    .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            }
                            None => Ok(crate::inst::operand::Zero::zero()),
                        }
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 3))?,
                    ),
                )
            }
            fn encode<
                E: crate::inst::Emitter,
                RL: crate::inst::Resolver<::lasso::Spur, u64>,
            >(&self, e: &mut E, rl: &mut RL) {
                use crate::inst::Emitter;
                use crate::inst::Resolver;
                use crate::inst::Encode;
                use crate::inst::IntN;
                e.begin_instr();
                {
                    let value: IntN = {
                        let value = &self.0;
                        IntN((value.size == crate::inst::operand::GprSize::B8) as u32, 1)
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        IntN(0b00100010, ("0b00100010".len() - 2) as u8)
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.3;
                        value.encode()
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.2;
                        value.encode()
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.1;
                        value.encode()
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.0;
                        value.encode()
                    };
                    e.push(value);
                }
                e.end_instr();
            }
        }
        #[allow(non_camel_case_types)]
        pub struct B_Default(crate::inst::operand::LabelSImm<26, 4>);
        impl From<B_Default> for InstrSet {
            fn from(value: B_Default) -> Self {
                InstrSet::B_Default(value)
            }
        }
        impl Instruction for B_Default {
            const MNEM: super::Mnemonic = super::Mnemonic::B;
            const VARIANT: Variant = Variant::Default;
            const ARGS: &'static [crate::inst::util::Param] = &[
                crate::inst::util::Param::Req(
                    <crate::inst::operand::LabelSImm<
                        26,
                        4,
                    > as crate::inst::operand::Operand>::KIND,
                ),
            ];
            #[allow(unused_mut, unused_variables)]
            fn from_args<'a, A, I: Iterator>(
                mut iter: I,
            ) -> ::std::result::Result<Self, crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: crate::inst::Arg,
                A: 'a,
            {
                Ok(
                    Self(
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 0))?,
                    ),
                )
            }
            fn encode<
                E: crate::inst::Emitter,
                RL: crate::inst::Resolver<::lasso::Spur, u64>,
            >(&self, e: &mut E, rl: &mut RL) {
                use crate::inst::Emitter;
                use crate::inst::Resolver;
                use crate::inst::Encode;
                use crate::inst::IntN;
                e.begin_instr();
                {
                    let value: IntN = { IntN(0b000101, ("0b000101".len() - 2) as u8) };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.0;
                        {
                            fn convert<T: crate::inst::Resolvable>(
                                _: &T,
                            ) -> fn(val: T::Value) -> IntN
                            where
                                T::Result: Encode,
                            {
                                |val| {
                                    (crate::inst::resolve_value_to_result::<T>(val)).encode()
                                }
                            }
                            crate::inst::resolve_or_push_fixup_opt(
                                value,
                                e,
                                rl,
                                value.0,
                                convert(value),
                            )
                        }
                    };
                    e.push(value);
                }
                e.end_instr();
            }
        }
        #[allow(non_camel_case_types)]
        pub struct B_Condition(
            crate::inst::operand::Cond,
            crate::inst::operand::LabelSImm<19, 4>,
        );
        impl From<B_Condition> for InstrSet {
            fn from(value: B_Condition) -> Self {
                InstrSet::B_Condition(value)
            }
        }
        impl Instruction for B_Condition {
            const MNEM: super::Mnemonic = super::Mnemonic::B;
            const VARIANT: Variant = Variant::Condition;
            const ARGS: &'static [crate::inst::util::Param] = &[
                crate::inst::util::Param::Req(
                    <crate::inst::operand::Cond as crate::inst::operand::Operand>::KIND,
                ),
                crate::inst::util::Param::Req(
                    <crate::inst::operand::LabelSImm<
                        19,
                        4,
                    > as crate::inst::operand::Operand>::KIND,
                ),
            ];
            #[allow(unused_mut, unused_variables)]
            fn from_args<'a, A, I: Iterator>(
                mut iter: I,
            ) -> ::std::result::Result<Self, crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: crate::inst::Arg,
                A: 'a,
            {
                Ok(
                    Self(
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 0))?,
                        iter
                            .next()
                            .ok_or(crate::inst::ParseError::Required)
                            .and_then(|value| crate::inst::Arg::parse(value))
                            .and_then(|op| crate::inst::operand::Operand::from_op(op))
                            .map_err(|e| crate::inst::ParseErrorIdx(e, 1))?,
                    ),
                )
            }
            fn encode<
                E: crate::inst::Emitter,
                RL: crate::inst::Resolver<::lasso::Spur, u64>,
            >(&self, e: &mut E, rl: &mut RL) {
                use crate::inst::Emitter;
                use crate::inst::Resolver;
                use crate::inst::Encode;
                use crate::inst::IntN;
                e.begin_instr();
                {
                    let value: IntN = {
                        IntN(0b01010100, ("0b01010100".len() - 2) as u8)
                    };
                    e.push(value);
                }
                {
                    let value: IntN = {
                        let value = &self.1;
                        {
                            fn convert<T: crate::inst::Resolvable>(
                                _: &T,
                            ) -> fn(val: T::Value) -> IntN
                            where
                                T::Result: Encode,
                            {
                                |val| {
                                    (crate::inst::resolve_value_to_result::<T>(val)).encode()
                                }
                            }
                            crate::inst::resolve_or_push_fixup_opt(
                                value,
                                e,
                                rl,
                                value.0,
                                convert(value),
                            )
                        }
                    };
                    e.push(value);
                }
                e.end_instr();
            }
        }
        #[allow(non_camel_case_types)]
        pub enum InstrSet {
            ADD_Immediate(ADD_Immediate),
            B_Default(B_Default),
            B_Condition(B_Condition),
        }
        pub fn emit_instr<
            E: crate::inst::Emitter,
            RL: crate::inst::Resolver<::lasso::Spur, u64>,
        >(instr: InstrSet, e: &mut E, rl: &mut RL) {
            match instr {
                InstrSet::ADD_Immediate(i) => i.encode(e, rl),
                InstrSet::B_Default(i) => i.encode(e, rl),
                InstrSet::B_Condition(i) => i.encode(e, rl),
            }
        }
        pub fn parse_variant<'a, I, A>(
            mnem: super::Mnemonic,
            iter: I,
        ) -> ::std::result::Result<Option<InstrSet>, crate::inst::ParseErrorIdx>
        where
            I: Iterator<Item = &'a A> + Clone,
            A: crate::inst::Arg,
            A: 'a,
        {
            match mnem {
                super::Mnemonic::ADD => {
                    if crate::inst::util::is_variant(iter.clone(), ADD_Immediate::ARGS) {
                        return Ok(
                            Some(<ADD_Immediate as Instruction>::from_args(iter)?.into()),
                        );
                    }
                    Ok(None)
                }
                super::Mnemonic::B => {
                    if crate::inst::util::is_variant(iter.clone(), B_Default::ARGS) {
                        return Ok(
                            Some(<B_Default as Instruction>::from_args(iter)?.into()),
                        );
                    }
                    if crate::inst::util::is_variant(iter.clone(), B_Condition::ARGS) {
                        return Ok(
                            Some(<B_Condition as Instruction>::from_args(iter)?.into()),
                        );
                    }
                    Ok(None)
                }
            }
        }
    }
}
