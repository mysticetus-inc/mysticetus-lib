use std::cell::Cell;
use std::ops::{self, Bound, Not};
use std::slice::SliceIndex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Visitor<'a, T> {
    items: &'a [T],
    visited: Box<[Cell<Visited>]>,
    next_unvisited: Cell<Option<usize>>,
    num_unvisited: Cell<usize>,
}

impl<'a, T> Visitor<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        let len = items.len();

        let mut visited = Vec::with_capacity(len);
        visited.resize(len, Cell::new(Visited::No));

        let next_unvisited = if len == 0 {
            Cell::new(None)
        } else {
            Cell::new(Some(0))
        };

        Self {
            items,
            visited: visited.into_boxed_slice(),
            next_unvisited,
            num_unvisited: Cell::new(len),
        }
    }

    pub fn num_visited(&self) -> usize {
        self.items.len() - self.num_unvisited.get()
    }

    pub fn all_visited(&self) -> bool {
        self.num_unvisited.get() == 0
    }

    pub fn next_unvisited_after(&self, start_at: usize) -> Option<&T> {
        if self.all_visited() || start_at >= self.items.len() {
            return None;
        }

        if let Some(next_unvisited) = self.next_unvisited.get() {
            if next_unvisited > start_at {
                return self.get(next_unvisited);
            }
        }

        let visited_subset = self.visited.get(start_at..)?;

        for (offset, visited) in visited_subset.iter().enumerate() {
            if visited.get().is_unvisited() {
                return self.get(start_at + offset);
            }
        }

        None
    }

    pub fn num_unvisited(&self) -> usize {
        self.num_unvisited.get()
    }

    fn mark_visited<I>(&self, visited: I)
    where
        I: VisitorIndex<T>,
    {
        let (start, end) = visited.bounds();

        let start_idx = match start {
            Bound::Included(idx) => idx,
            Bound::Excluded(excl) => excl + 1,
            Bound::Unbounded => 0,
        };

        let end_idx = match end {
            Bound::Included(incl) => incl + 1,
            Bound::Excluded(excl) => excl,
            Bound::Unbounded => self.items.len(),
        };

        let subset = match self.visited.get(start_idx..end_idx) {
            Some(subset) => subset,
            None => return,
        };

        if let Some(remainder) = self
            .next_unvisited
            .take()
            .filter(|next_unvisited| (start_idx..end_idx).contains(next_unvisited))
            .and_then(|_| self.items.get(end_idx..))
        {
            for (offset, _) in remainder.iter().enumerate() {
                if let Some(Visited::No) = self.visited.get(end_idx + offset).map(Cell::get) {
                    self.next_unvisited.set(Some(end_idx + offset));
                    break;
                }
            }
        }

        let mut visited_count = 0;
        for visited in subset {
            if visited.replace(Visited::Yes).is_unvisited() {
                visited_count += 1;
            }
        }

        let prev = self.num_unvisited.get();
        self.num_unvisited.set(prev.saturating_sub(visited_count));
    }

    pub fn get<I>(&self, range: I) -> Option<&I::Output>
    where
        I: VisitorIndex<T>,
    {
        let output = self.items.get(range.clone())?;
        self.mark_visited(range);
        Some(output)
    }

    pub fn next_unvisited(&self) -> Option<&T> {
        if self.all_visited() {
            return None;
        }

        if let Some(index) = self.next_unvisited.take() {
            return self.get(index);
        }

        for (idx, visited) in self.visited.iter().enumerate() {
            if visited.get().is_unvisited() {
                return self.get(idx);
            }
        }

        None
    }

    pub fn iter_unvisited(&self) -> UnvisitedIter<'_, T> {
        UnvisitedIter {
            iter: self.items.iter().enumerate(),
            visitor: self,
        }
    }

    pub fn next_unvisited_with<F>(&self, mut f: F) -> Option<&T>
    where
        F: FnMut(&T) -> bool,
    {
        if self.all_visited() {
            return None;
        }

        let start_offset = self.next_unvisited.take().unwrap_or(0);

        for (offset, visited) in self.visited.iter().enumerate() {
            if visited.get().has_visited() {
                continue;
            }

            let index = start_offset + offset;

            let item = &self.items[index];

            if f(item) {
                self.mark_visited(index);
                return Some(item);
            } else if self.next_unvisited.get().is_none() {
                self.next_unvisited.set(Some(index));
            }
        }

        None
    }

    pub fn peek_next_unvisited_with<F>(&self, mut f: F) -> Option<Peek<'_, T>>
    where
        F: FnMut(&T) -> bool,
    {
        let start_idx = self.next_unvisited.take().unwrap_or(0);

        let rem = self.visited.get(start_idx..)?;

        for (offset, visited) in rem.iter().enumerate() {
            let index = start_idx + offset;
            if visited.get().has_visited() {
                continue;
            } else if self.next_unvisited.get().is_none() {
                self.next_unvisited.set(Some(index));
            }

            let item = &self.items[index];
            if f(item) {
                return Some(Peek {
                    item,
                    visited: self,
                    index,
                });
            }
        }

        None
    }

    pub fn peek(&self, index: usize) -> Option<Peek<'_, T>> {
        let item = self.items.get(index)?;

        Some(Peek {
            item,
            index,
            visited: self,
        })
    }
}

pub struct Peek<'a, T> {
    item: &'a T,
    visited: &'a Visitor<'a, T>,
    index: usize,
}

impl<'a, T> Peek<'a, T> {
    pub fn visit(self) -> &'a T {
        self.visited.mark_visited(self.index);
        self.item
    }

    #[inline]
    pub const fn peek(&self) -> &T {
        self.item
    }

    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    pub fn visited(&self) -> Visited {
        self.visited.visited[self.index].get()
    }

    pub fn has_visited(&self) -> bool {
        self.visited().has_visited()
    }

    pub fn is_unvisited(&self) -> bool {
        self.visited().is_unvisited()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visited {
    Yes,
    No,
}

impl Not for Visited {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Yes => Self::No,
            Self::No => Self::Yes,
        }
    }
}

impl Visited {
    /// Shortcut for `x == Visited::No`
    /// ```
    /// # use data_structures::visitor::Visited;
    /// assert!(!Visited::Yes.is_unvisited());
    /// assert!(Visited::No.is_unvisited());
    /// ```
    #[inline]
    pub const fn is_unvisited(&self) -> bool {
        matches!(*self, Self::No)
    }

    /// Shortcut for `x == Visit::Yes`
    /// ```
    /// # use data_structures::visitor::Visited;
    /// assert!(Visited::Yes.has_visited());
    /// assert!(!Visited::No.has_visited());
    /// ```
    #[inline]
    pub const fn has_visited(&self) -> bool {
        !self.is_unvisited()
    }
}

pub trait VisitorIndex<T>: SliceIndex<[T]> + Clone {
    fn bounds(&self) -> (Bound<usize>, Bound<usize>);
}

impl<T> VisitorIndex<T> for usize {
    fn bounds(&self) -> (Bound<usize>, Bound<usize>) {
        (Bound::Included(*self), Bound::Included(*self))
    }
}

impl<T, R> VisitorIndex<T> for R
where
    R: SliceIndex<[T]> + ops::RangeBounds<usize> + private::Sealed + Clone,
{
    fn bounds(&self) -> (Bound<usize>, Bound<usize>) {
        let start = ops::RangeBounds::start_bound(self).cloned();
        let end = ops::RangeBounds::end_bound(self).cloned();
        (start, end)
    }
}

impl private::Sealed for ops::Range<usize> {}
impl private::Sealed for ops::RangeFrom<usize> {}
impl private::Sealed for ops::RangeTo<usize> {}
impl private::Sealed for ops::RangeToInclusive<usize> {}
impl private::Sealed for ops::RangeInclusive<usize> {}
impl private::Sealed for ops::RangeFull {}

mod private {
    pub trait Sealed {}
}

pub struct UnvisitedIter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, T>>,
    visitor: &'a Visitor<'a, T>,
}

impl<'a, T> Iterator for UnvisitedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (idx, curr) = self.iter.next()?;

            if self.visitor.visited[idx].get().is_unvisited() {
                self.visitor.mark_visited(idx);
                return Some(curr);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.visitor.num_unvisited.get();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for UnvisitedIter<'_, T> {}

impl<T> DoubleEndedIterator for UnvisitedIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (idx, curr) = self.iter.next_back()?;

            if self.visitor.visited[idx].get().is_unvisited() {
                self.visitor.mark_visited(idx);
                return Some(curr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor() {
        const DATA: &[i32] = &[0, 1, 2, 3, 4, 5];
        let visitor = Visitor::new(DATA);

        // make sure initial counts are right
        assert_eq!(visitor.num_visited(), 0);
        assert_eq!(visitor.num_unvisited(), DATA.len());

        // run over this twice, to make sure a given element isnt counted as 'visited' twice
        for _ in 0..2 {
            assert_eq!(visitor.get(0).unwrap(), &0);

            assert_eq!(visitor.num_visited(), 1);
            assert_eq!(visitor.num_unvisited(), DATA.len() - 1);
        }

        assert_eq!(visitor.get(4..).unwrap(), &[4, 5]);

        assert_eq!(visitor.num_visited(), 3);
        assert_eq!(visitor.num_unvisited(), 3);

        let peeked = visitor.peek(2).unwrap();

        assert_eq!(peeked.peek(), &2);
        assert!(peeked.is_unvisited());
        assert_eq!(peeked.visit(), &2);

        let not_consumed_peek = visitor.peek(1).unwrap();

        assert_eq!(not_consumed_peek.peek(), &1);
        assert!(not_consumed_peek.is_unvisited());

        assert_eq!(visitor.iter_unvisited().copied().collect::<Vec<_>>(), vec![
            1, 3
        ]);

        assert!(visitor.all_visited());
    }
}
