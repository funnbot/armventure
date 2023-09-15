mod sf {
    trait SizeField {
        const BYTES: usize;
        const BITS: u32;
    }

    struct Sf32;
    impl SizeField for Sf32 {
        const BYTES: usize = 4;
        const BITS: u32 = 32;
    }

    struct Sf64;
    impl SizeField for Sf64 {
        const BYTES: usize = 8;
        const BITS: u32 = 64;
    }
}
