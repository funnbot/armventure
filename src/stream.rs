pub trait Stream {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
    fn peek(&self) -> Option<&Self::Item>;

    fn next_if_eq<I>(&mut self, item: I) -> Option<Self::Item>
    where
        Self::Item: PartialEq<I>,
    {
        if *self.peek()? == item {
            self.next()
        } else {
            None
        }
    }
    fn next_if<Pred>(&mut self, pred: Pred) -> Option<Self::Item>
    where
        Pred: FnOnce(&Self::Item) -> bool,
    {
        if pred(self.peek()?) {
            self.next()
        } else {
            None
        }
    }
    fn next_while<Pred>(&mut self, mut pred: Pred) -> usize
    where
        Pred: FnMut(&Self::Item) -> bool,
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
    fn map_if<T, Pred>(&mut self, pred: Pred) -> Option<T>
    where
        Pred: FnOnce(&Self::Item) -> Option<T>,
    {
        let output = pred(self.peek()?)?;
        self.next();
        Some(output)
    }
}

pub trait MultiPeek: Stream {
    fn peek_by(&self, n: usize) -> Option<Self::Item>;
}

// pub trait AdapterInner<S: Stream> {
//     fn get_inner(&self) -> &S;
// }
// pub trait AdapterInnerMut<S: Stream> {
//     fn get_inner(&mut self) -> &mut S;
// }
// impl<S: Stream + AdapterInnerMut<S>> AdapterInner<S> for S {
//     fn get_inner(&self) -> &S {
//         self.get_inner()
//     }
// }
pub trait AdapterToInner<S: Stream> {
    fn to_inner(self) -> S;
}
pub trait IterAsInner<I: Iterator> {
    fn to_inner(self) -> I;
}
impl<'s> IterAsInner<std::slice::Iter<'s, u8>> for std::str::CharIndices<'s> {
    fn to_inner(self) -> std::slice::Iter<'s, u8> {
        self.as_str().as_bytes().iter()
    }
}
