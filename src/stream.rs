// <https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats>
pub trait PeekLifetime<'this, ImplicitBounds: Sealed = Bounds<&'this Self>> {
    type PeekItem: Copy;
}

mod sealed {
    pub trait Sealed: Sized {}
    pub struct Bounds<T>(T);
    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};

pub type PeekItem<'this, T> = <T as PeekLifetime<'this>>::PeekItem;

pub trait Stream: for<'this> PeekLifetime<'this> {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
    fn peek(&self) -> Option<PeekItem<'_, Self>>;

    fn next_if_eq<I>(&mut self, item: I) -> Option<Self::Item>
    where
        for<'this> PeekItem<'this, Self>: PartialEq<I>,
    {
        if self.peek()? == item {
            self.next()
        } else {
            None
        }
    }
    fn next_if<Pred>(&mut self, pred: Pred) -> Option<Self::Item>
    where
        Pred: FnOnce(PeekItem<'_, Self>) -> bool,
    {
        if pred(self.peek()?) {
            self.next()
        } else {
            None
        }
    }
    fn next_while<Pred>(&mut self, mut pred: Pred) -> usize
    where
        Pred: FnMut(PeekItem<'_, Self>) -> bool,
    {
        let mut count: usize = 0;
        while let Some(item) = self.peek() {
            if pred(item) {
                count += 1;
                self.next();
            } else {
                break;
            }
        }
        count
    }
    fn advance_by(&mut self, mut n: usize) -> usize {
        while n > 0 {
            debug_assert_ne!(n, 0);
            if self.next().is_some() {
                break;
            }
            n -= 1;
        }
        n
    }
}

// TODO:
// macro_rules! impl_stream_for {
//     ($type:ty) => {
//         impl<'this, 's> $crate::
//         impl $crate::Stream for $type
//     };
// }

// impl<'this, 's> PeekLifetime<'this> for SourceIterator<'s> {
//     type PeekItem = char;
// }

// impl<'s> Stream for SourceIterator<'s> {
//     type Item = char;

//     fn peek(&self) -> Option<char> {
//         self.current
//     }
//     fn next(&mut self) -> Option<char> {
//         let c = self.peek()?;
//         self.current = self.inner.next();

//         // works with \r\n
//         if c == '\n' {
//             self.location.inc_line();
//         } else {
//             self.location.inc_column();
//         }
//         Some(c)
//     }
// }

pub trait MultiPeek: Stream
where
    Self::Item: Clone,
{
    fn peek_by(&self, n: usize) -> Option<Self::Item>;
}
