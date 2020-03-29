// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    storage2 as storage,
    storage2::{
        collections::extend_lifetime,
        PullForward,
        SaturatingStorage,
        StorageFootprint,
    },
};

/// An iterator over shared references to the elements of a storage stash.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T> {
    /// The storage stash to iterate over.
    stash: &'a storage::Stash<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Iter<'a, T> {
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a storage::Stash<T>) -> Self {
        Self {
            stash,
            begin: 0,
            end: stash.len(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: StorageFootprint + PullForward + scale::Decode,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        let cur = self.begin;
        self.begin += 1;
        self.stash.get(cur)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where
    T: StorageFootprint + PullForward + scale::Decode
{
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: StorageFootprint + PullForward + scale::Decode,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        self.stash.get(self.end)
    }
}

/// An iterator over exclusive references to the elements of a storage stash.
#[derive(Debug)]
pub struct IterMut<'a, T> {
    /// The storage stash to iterate over.
    stash: &'a mut storage::Stash<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> IterMut<'a, T> {
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a mut storage::Stash<T>) -> Self {
        let len = stash.len();
        Self {
            stash,
            begin: 0,
            end: len,
        }
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Decode,
{
    fn get_mut<'b>(&'b mut self, at: u32) -> Option<&'a mut T> {
        self.stash.get_mut(at).map(|value| {
            // SAFETY: We extend the lifetime of the reference here.
            //
            //         This is safe because the iterator yields an exclusive
            //         reference to every element in the iterated vector
            //         just once and also there can be only one such iterator
            //         for the same vector at the same time which is
            //         guaranteed by the constructor of the iterator.
            unsafe { extend_lifetime::<'b, 'a, T>(value) }
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Decode,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        let cur = self.begin;
        self.begin += 1;
        self.get_mut(cur)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Decode
{
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: StorageFootprint + SaturatingStorage + PullForward + scale::Decode,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        self.get_mut(self.end)
    }
}