// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Uniq iterator
//!
//! Filters out duplicate elements of an iterator.

use std::collections::BTreeSet;


pub struct UniqIterator<I, T> {
    source: I,
    // TODO: HashSet would let me eliminate the Ord requirement; I haven't
    // needed it yet though. When I do, I'd like to implicitly switch.
    seen: BTreeSet<T>,
}

impl<I, T> Iterator for UniqIterator<I, T>
    where I: Iterator + Iterator<Item = T>,
          T: Copy + Ord,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let seen = &mut self.seen;

        match self.source.find(|i| { ! seen.contains(i) }) {
            Some(i) => {
                seen.insert(i);
                Some(i)
            },
            None => None,
        }
    }
}

pub trait Uniq<I, T>: Iterator {
    fn uniq(self) -> UniqIterator<Self, T>
        where Self: Sized + Iterator<Item = T>,
              T: Ord,
    {
        UniqIterator {
            source: self,
            seen: BTreeSet::new(),
        }
    }
}

impl<I, T> Uniq<I, T> for I where I: Iterator {}
