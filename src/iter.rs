use std::collections::VecDeque;

use itertools::PeekingNext;

pub struct NanoPeek<I: Iterator> {
    pub inner: I,
    buf: VecDeque<I::Item>,
    idx: usize,
}

impl<I: Iterator> NanoPeek<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            buf: VecDeque::new(),
            idx: 0,
        }
    }
    /// Reset the peeking “cursor”
    pub fn reset_peek(&mut self) {
        self.idx = 0;
    }
}

impl<I: Iterator> NanoPeek<I> {
    pub fn peek(&mut self) -> Option<&I::Item> {
        let ret = if self.idx < self.buf.len() {
            Some(&self.buf[self.idx])
        } else {
            match self.inner.next() {
                Some(x) => {
                    self.buf.push_back(x);
                    Some(&self.buf[self.idx])
                }
                None => return None,
            }
        };

        self.idx += 1;
        ret
    }
}

impl<I> PeekingNext for NanoPeek<I>
where
    I: Iterator,
{
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool,
    {
        if self.buf.is_empty() {
            if let Some(r) = self.peek() {
                if !accept(r) {
                    return None;
                }
            }
        } else if let Some(r) = self.buf.get(0) {
            if !accept(r) {
                return None;
            }
        }
        self.next()
    }
}

impl<I> Iterator for NanoPeek<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx = 0;
        self.buf.pop_front().or_else(|| self.inner.next())
    }
}

// Same size
impl<I> ExactSizeIterator for NanoPeek<I> where I: ExactSizeIterator {}
