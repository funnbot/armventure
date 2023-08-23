macro_rules! stringify2 {
    ($literal:literal) => {
        $literal
    };
    ($tt:tt) => {
        stringify!($tt)
    };
}

macro_rules! comp_error {
    ($($tt:tt),+) => {
        compile_error!(concat!(
            $(stringify2!($tt)),+
        ))
    };
}

macro_rules! __def_mnemonic {
    {$( $mnem:ident )+} => {
        pub enum Mnemonic {
            $( $mnem ),+
        }
        impl std::str::FromStr for Mnemonic {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let hash = $crate::inst::util::str_hash(s);
                $(
                if $crate::inst::util::str_hash( stringify!( $mnem ) ) == hash {
                    return Ok( Mnemonic:: $mnem );
                }
                )+
                Err(())
            }
        }
    }
}

macro_rules! __def_inst_type {
    {
        $mnem:ident $inst:ident $( $variant:ident )?
        ( $( $arg_name:ident $arg_opts:tt )* )
        ( $( $enc_name:ident $enc_opts:tt $( : $enc_idx:tt )? )* )
    } => {
        #[allow(non_camel_case_types)]
        pub struct $inst(
        $(
            _arg_type!($arg_name $arg_opts)
        ),*
        );
        impl From<$inst> for InstrSet {
            fn from(value: $inst) -> Self {
                InstrSet::$inst(value)
            }
        }
        impl Instruction for $inst {
            const MNEM: super::Mnemonic = super::Mnemonic:: $mnem;
            $( const VARIANT: Variant = Variant:: $variant; )?
            const ARGS: &'static [$crate::inst::util::Param] = &[
        $(
                _arg_kind!($arg_name $arg_opts)
        ),*
            ];
            #[allow(unused_mut, unused_variables)]
            fn from_args<'a, A, I: Iterator>(mut iter: I) -> ::std::result::Result<Self, $crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: $crate::inst::Arg,
                A: 'a,
            {
                Ok(Self(
        $(
                    _arg_parse!($arg_name iter)
                        .map_err(|e| $crate::inst::ParseErrorIdx(e, ${index()}))?
        ),*
                ))
            }
            fn encode<E: $crate::inst::Emitter>(&self, e: &mut E) {
                use $crate::inst::Emitter;
                use $crate::inst::Encode;
                use $crate::inst::operand::Encoder;
                use $crate::inst::IntN;
                e.begin_instr();
                $(
                    {
                        $( let value = &self.$enc_idx; )?
                        _arg_encode!($enc_name $enc_opts value e)
                    }
                    // {
                    //     let value = {
                    //         $( let value = &self.$enc_idx; )?
                    //         _arg_encode!($enc_name $enc_opts value e)
                    //     };
                    //     e.push(value.encode());
                    // }
                )*
                e.end_instr();
            }
        }
    };
}

macro_rules! __def_insts {
    {
    $(
        $mnem:ident :
            $(
                $inst:ident $( $variant:ident )?
                ( $( $args:tt )* )
                ( $( $encode:tt )* )
            ),+
        ;
    )+
    } => {
        $(  $( // mnem, variant
            __def_inst_type!{
                $mnem $inst $($variant)? ( $($args)* ) ( $($encode)* )
            }
        )+  )+

        #[allow(non_camel_case_types)]
        pub enum InstrSet {
        $(  $(
            $inst($inst),
        )+  )+
        }

        pub fn emit_instr<E: $crate::inst::Emitter>(instr: InstrSet, e: &mut E) {
            match instr {
        $(  $(
                InstrSet::$inst(i) => i.encode(e),
        )+  )+
            }
        }

        pub fn parse_variant<'a, I, A>(mnem: super::Mnemonic, iter: I) -> ::std::result::Result<Option<InstrSet>, $crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A> + Clone,
                A: $crate::inst::Arg,
                A: 'a
        {
            match mnem {
        $( // mnem
                super::Mnemonic:: $mnem => {
            $( // variant
                    if $crate::inst::util::is_variant(iter.clone(), $inst :: ARGS) {
                        return Ok(Some(<$inst as Instruction>::from_args(iter)?.into()))
                    }
            )+
                    Ok(None)
                }
        ),+
            }
        }
    }
}

macro_rules! def_instrs {
    {
    $(
        $mnem:ident $(
            $( $variant:ident )?
            ( $( $args:tt )* )
            ( $( $encode:tt )* )
        ),+ ;
    )+
    } => {
        __def_mnemonic!{ $( $mnem )+ }

        pub mod parse {
        use $crate::inst::meta::*;
        use super::Variant;

        pub trait Instruction
        where
            Self: Sized,
        {
            const MNEM: super::Mnemonic;
            const VARIANT: Variant = Variant::Default;
            const ARGS: &'static [$crate::inst::util::Param];

            fn from_args<'a, A, I: Iterator>(iter: I) -> ::std::result::Result<Self, $crate::inst::ParseErrorIdx>
            where
                I: Iterator<Item = &'a A>,
                A: $crate::inst::Arg,
                A: 'a;
            fn encode<E: $crate::inst::Emitter>(&self, e: &mut E);
        }

        paste! {
            __def_insts!{
                $(
                    $mnem : $(
                        [<$mnem $(_ $variant)?>] $($variant)?
                        ( $($args)* )
                        ( $($encode)* )
                    ),+ ;
                )+
            }
        }

        }
    }
}

pub(super) use super::meta_operand::*;
pub(super) use {
    __def_inst_type, __def_insts, __def_mnemonic, comp_error, def_instrs, paste::paste, stringify2,
};
