/*!
## extended syntax
```
#[adt]
enum Enum {
    MyNested enum {

    },
}
#[adt]
struct Struct {
    my_nested_enum: enum {

    }
    my_nested_struct: struct {

    }
}
```

## attribute options
`marker(trait, ..)`
- auto derive empty marker traits for all types in adt

`prop(ident, ..)` | `prop_get(ident, ..)` | `prop_set(ident, ..)`
- add accessors for shared enum fields, if all options have the property returns T, otherwise returns Option<T>
- only allowed on enum

`in path` | `in path::*`
- set the name of the module for the nested items, if not set, uses the item name converted to lower camel case
- the ::* at the end will add `use path::*;` at the modules level, to bring the adt's items into scope.

`as Ident`
- renames the generated type for that field, rather than using the field name


## Examples
```
#[adt(marker(MyTrait), props(bar), in foo)]
#[derive(Debug)] // must be after due to non-standard syntax, applies to all generated types
enum Foo<'a, T, I: Iterator>
where I::Item: T,
{
    Struct {
        foo: u8,
        bar: &'a T,
    }
    Tuple(u8, u8),
    #[adt in nested::*]
    Nested enum {
        One,
        Two(u8),
        Three {
           bar: u8
        }
    }
    #[adt in inner]
    InnerStruct {
        baz: struct Baz {

        }
    }
    BoundedInteger {
        #[adt(range(0..4))]
        int: u8
    }
    Bar enum {

    }
    #[adt(subset(Self::Bar, A, !B), union(Self::Bar2))]
    Baz enum {
    }
}
```
becomes
```
mod foo {
   struct Struct<'a, T> {
       foo: u8,
       bar: &'a T,
   }
   struct Tuple(u8, u8);
   mod nested {
       struct One();
       struct Two(u8);
       struct Three {
           bar: u8
       }
   }
   use nested::*;
   enum Nested {
       One(nested::One),
       Two(nested::Two),
       Three(nested::Three),
   }
   mod inner {
       struct Baz {
       }
   }
   struct InnerStruct {

   }
}
enum Foo<'a, T, I>
where I: Iterator,
      I::Item: T,
{
    #[doc = "foo::Struct<'a, T> {
foo: u8,
}"]
    Struct(foo::Struct<'a, T>);
    Tuple(foo::Tuple);
    Nested(foo::Nested);
}
```

```
//! const SOME_N: u8 = 4;
#[adt in also_works]
struct AlsoWorks<T, const N: u8> {
    #[adt(range(0..5, 10, !2)) as IntRange]
    int_range: u8,
    #[adt(size(u2, !0))]
    int_bits: u16,
    #[adt(range(-1..=N))]
    generic_range: u8,
    #[adt(range(1..{ N - SOME_N }))]
    generic_range: u8,
    #[adt(size(T))]
    generic_bits: u8

    nested_enum: enum {

    }
    nested_struct: struct {

    }
}
```
becomes
```
mod also_works {
    // if the enum would have > 256 values, it will instead be a struct
    #[repr(u8)]
    enum IntRange {
        Z = 0, P1 = 1, P3 = 3, P4 = 4, P10 = 10,
    }
    #[repr(u8)]
    enum int_bits {
        P1 = 1, P2 = 2, P3 = 3
    }
    struct generic_range<const N: u8> {
        value: u8
    }
    impl<const N: u8> TryFrom<u8> for generic_range<N> {
        fn try_from(value: u8) -> Option<Self> {
            match value {
                -1..N => Self { value },
                _ => None,
            }
        }
    }
}
```

```
#[adt(size(u14))]
struct BoundedInteger(u16);
```
*/
pub use adt_internal_items::BoundedInteger;
pub use adt_internal_macros::adt;
