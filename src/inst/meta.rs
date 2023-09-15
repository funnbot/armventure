macro_rules! __def_mnemonic {
    {$( $mnem:ident )+} => {
        $crate::enum_str! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Mnemonic {
                $( $mnem ),+
            }
        }
    };
}

macro_rules! _def_dec_instr {
    {
        $mnem:ident $inst:ident $( $variant:ident )?
        ( $( $arg_name:ident $arg_opts:tt )* )
        ( $( $enc_name:ident $enc_opts:tt $( : $enc_idx:tt )? )* )
    } => {
        #[allow(non_camel_case_types)]
        pub struct $inst (u32);
    };
}

macro_rules! __def_inst_type {
    {
        $mnem:ident $inst:ident $( $variant:ident )?
        ( $( $arg_name:ident $arg_opts:tt )* )
        ( $( $enc_name:ident $enc_opts:tt $( : $enc_idx:tt )? )* )
    } => {
        #[allow(non_camel_case_types)]
        pub struct $inst (
            $( _arg_type!($arg_name $arg_opts) ),*
        );
        impl super::EncInstr for $inst {
            const MNEM: super::Mnemonic = super::Mnemonic:: $mnem;
            $( const VARIANT: super::Variant = super::Variant:: $variant; )?
            const ARGS: &'static [$crate::inst::util::Param] = &[
                $( _arg_kind!($arg_name $arg_opts) ),*
            ];

            fn from_ops<'a, I>(mut iter: I) -> Self
            where
                I: ::std::iter::Iterator<Item = &'a $crate::inst::operand::Ops>
            {
                Self(
                    $( _arg_parse!($arg_name iter) ),*
                )
            }
            fn emit<E: $crate::inst::Emitter>(&self, e: &mut E) -> ::std::result::Result<(), $crate::inst::ErrorMacro> {
                use $crate::inst::operand::Encoder;
                use ::bit::IntN;
                use $crate::inst::operand::enc;
                e.begin_instr();
                $(
                    _arg_encode!( $enc_name $enc_opts self e $( $enc_idx )? )
                        .map_err(|e| $crate::inst::ErrorMacro(e,
                            concat!( stringify!($enc_name), stringify!($enc_opts) $(, stringify!( : $enc_idx ) )? )
                        ))?;
                )*
                e.end_instr();
                Ok(())
            }
        }
        impl ::std::fmt::Debug for $inst {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", stringify!( $mnem ))?;
                $( write!(f, "({}) ", stringify!( $variant ))?; )?
                $crate::write_join!(f, "{:?}", ", " $(, ${ignore(arg_name)} self. ${index()} )* )
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

        pub trait EncInstr
        where
            Self: Sized + ::std::fmt::Debug,
        {
            const MNEM: Mnemonic;
            const VARIANT: Variant = Variant::Default;
            const VARIANTS: &'static [Variant] = &[Self::VARIANT];
            const ARGS: &'static [$crate::inst::util::Param];

            fn from_ops<'a, I>(iter: I) -> Self
            where
                I: ::std::iter::Iterator<Item = &'a $crate::inst::operand::Ops>;
            fn emit<E: $crate::inst::Emitter>(&self, e: &mut E) -> ::std::result::Result<(), $crate::inst::ErrorMacro>;
        }

        pub mod enc {
            use $crate::inst::meta::*;

            $(  $( // mnem, variant
                __def_inst_type!{ $mnem $inst $($variant)? ( $($args)* ) ( $($encode)* ) }
            )+  )+
        }

        pub mod dec {
            use $crate::inst::meta::*;

            $( $(
                _def_dec_instr!{ $mnem $inst $($variant)? ( $($args)* ) ( $($encode)* ) }
            )+ )+
        }

        #[allow(non_camel_case_types)]
        pub enum EncInstrSet {
            $(  $(
                $inst($crate::inst::def::enc:: $inst),
            )+  )+
        }

        pub fn narrow_variant(mnem: Mnemonic) -> $crate::inst::NarrowVariant {
            match mnem {
            $(
                Mnemonic:: $mnem => {
                    $( debug_assert!($crate::inst::util::rest_are_opt(enc:: $inst :: ARGS)); )+
                    $crate::inst::NarrowVariant::new(&[
                        $( enc:: $inst :: ARGS ),+
                    ])
                }
            ),+
            }
        }

        pub fn get_variant_and_emit<'a, I, E>(mnem: Mnemonic, variant_idx: usize, iter: I, e: &mut E)
            -> ::std::result::Result<(), $crate::inst::ErrorMacro>
            where
                E: $crate::inst::Emitter,
                I: Iterator<Item = &'a $crate::inst::operand::Ops> + Clone,
            {
            match mnem {
            $(
                Mnemonic:: $mnem => {
                    match variant_idx {
                    $(
                        ${index()} => {
                            let instr = <$crate::inst::def::enc:: $inst as EncInstr>::from_ops(iter.clone());
                            println!("{:#010X}: {:?}", e.pc(), instr);
                            return instr.emit(e);
                        }
                    ),+
                        _ => ::std::unreachable!(),
                    }
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

        ::paste::paste! {
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

macro_rules! def_directs {
    { $( $name:ident ( $( $arg_name:ident $arg_opts:tt ),* ) ),* $(,)? } => {
        enum_str! {
            #[allow(non_camel_case_types)]
            pub enum Name {
                $( $name, )*
            }
        }

        pub fn narrow_variant(name: Name) -> $crate::inst::NarrowVariant {
            match name {
            $(
                Name:: $name => {
                    const EXPECT: &'static [$crate::inst::util::Param] =
                        &[ $( $crate::inst::meta_operand::_arg_kind!($arg_name $arg_opts) )* ];

                    debug_assert!($crate::inst::util::rest_are_opt(EXPECT));

                    $crate::inst::NarrowVariant::new(&[ EXPECT ])
                }
            ),+
            }
        }

        pub fn select_and_run<'a, E: Emitter, I>(e: &mut E, mut iter: I) -> std::result::Result<(), $crate::inst::dir::Error>
            where I: Iterator<Item = &'a Ops> + Clone,
        {
            $(
                {
                    const EXPECT: &'static [$crate::inst::util::Param] =
                        &[ $( $crate::inst::meta_operand::_arg_kind!($arg_name $arg_opts) )* ];
                    if $crate::inst::util::is_variant(iter.clone(), EXPECT) {
                        return $crate::inst::dir::def:: $name (e,
                            $( $crate::inst::meta_operand::_arg_parse!($arg_name iter), )*
                        );
                    }
                }
            )*
            todo!()
        }
    }
}

pub(super) use super::meta_operand::*;
pub(super) use __def_inst_type;
pub(super) use __def_insts;
pub(super) use __def_mnemonic;
pub(super) use _def_dec_instr;
pub(super) use def_directs;
pub(super) use def_instrs;
