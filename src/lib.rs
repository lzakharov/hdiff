//! # HDiff
//!
//! `hdiff` contains implementation of the Paul Heckel diff algorithm.

use std::collections::HashMap;
use std::vec::Vec;
use std::hash::Hash;

pub use self::Patch::*;

/// Finds difference between two slices of objects using Paul Heckel's algorithm.
///
/// # Examples
///
/// Example from the ["A technique for isolating differences between files"](https://dl.acm.org/doi/10.1145/359460.359467).
///
/// ```
/// use hdiff::*;
///
/// let old = vec!["MUCH", "WRITING", "IS", "LIKE", "SNOW", ",",
///                "A", "MASS","OF", "LONG", "WORDS", "AND",
///                "PHRASES", "FALLS", "UPON", "THE", "RELEVANT",
///                "FACTS", "COVERING", "UP", "THE", "DETAILS", "."];
/// let new = vec!["A", "MASS", "OF", "LATIN", "WORDS", "FALLS",
///                "UPON", "THE", "RELEVANT", "FACTS", "LIKE", "SOFT",
///                "SNOW", ",", "COVERING", "UP", "THE", "DETAILS", "."];
///
/// let answer = vec![Delete(0), Delete(1), Delete(2), Delete(9),
///                   Delete(11), Delete(12), Move(6, 0), Move(7, 1),
///                   Move(8, 2), Create(3), Move(10, 4), Move(13, 5),
///                   Move(14, 6), Move(15, 7), Move(16, 8), Move(17, 9),
///                   Move(3, 10), Create(11), Move(4, 12), Move(5, 13)];
///
/// assert_eq!(answer, diff(&old, &new));
/// ```
pub fn diff<T: Eq + Hash>(old: &[T], new: &[T]) -> Difference {
    let mut table: Table<&T> = HashMap::new();
    let mut na: Array = vec![None; new.len()];
    let mut oa: Array = vec![None; old.len()];

    pass1(new, &mut table);
    pass2(old, &mut table);
    pass3(new, table, &mut oa, &mut na);
    pass4(old, new, &mut oa, &mut na);
    pass5(old, new, &mut oa, &mut na);
    pass6(old, new, oa, na)
}

/// Contains patches between two slices of objects.
pub type Difference = Vec<Patch>;

/// Represents patch between two slices of objects.
#[derive(Debug, PartialEq)]
pub enum Patch {
    Create(usize),
    Update(usize),
    Move(usize, usize),
    Delete(usize),
}

type Table<T> = HashMap<T, TableEntry>;

struct TableEntry {
    nc: usize,
    oc: usize,
    olno: usize,
}

type Array = Vec<Entry>;

type Entry = Option<usize>;

fn pass1<'a, T: Eq + Hash>(new: &'a [T], table: &mut Table<&'a T>) {
    for x in new.iter() {
        match table.get_mut(x) {
            Some(tx) => {
                tx.nc += 1;
            },
            None => {
                table.insert(x, TableEntry{nc: 1, oc: 0, olno: 0});
            }
        }
    }
}

fn pass2<'a, T: Eq + Hash>(old: &'a [T], table: &mut Table<&'a T>) {
    for (i, x) in old.iter().enumerate() {
        match table.get_mut(x) {
            Some(tx) => {
                tx.oc += 1;
                tx.olno = i;
            },
            None => {
                table.insert(x, TableEntry{nc: 0, oc: 1, olno: i});
            }
        }
    }
}

fn pass3<T: Eq + Hash>(new: &[T], table: Table<&T>, oa: &mut Array, na: &mut Array) {
    for i in 0..na.len() {
        let tx = &table[&new[i]];
        if tx.nc == 1 && tx.nc == tx.oc {
            na[i] = Some(tx.olno);
            oa[tx.olno] = Some(i);
        }
    }
}

fn pass4<T: Eq>(old: &[T], new: &[T], oa: &mut Array, na: &mut Array) {
    if na.is_empty() {
        return;
    }

    for i in 0..na.len() - 1 {
        if let Some(j) = na[i] {
            if j + 1 < oa.len()
                && na[i + 1].is_none()
                && oa[j + 1].is_none()
                && new[i + 1] == old[j + 1]
            {
                na[i + 1] = Some(j + 1);
                oa[j + 1] = Some(i + 1);
            }
        }
    }
}

fn pass5<T: Eq>(old: &[T], new: &[T], oa: &mut Array, na: &mut Array) {
    for i in (1..na.len()).rev() {
        if let Some(j) = na[i] {
            if j > 0
                && na[i - 1].is_none()
                && oa[j - 1].is_none()
                && new[i - 1] == old[j - 1]
            {
                na[i - 1] = Some(j - 1);
                oa[j - 1] = Some(i - 1);
            }
        }
    }
}

fn pass6<T: Eq>(old: &[T], new: &[T], oa: Array, na: Array) -> Difference {
    let mut result = Difference::new();
    let mut delete_offsets = Vec::with_capacity(oa.len());
    let mut offset = 0;

    for (i, x) in oa.iter().enumerate() {
        delete_offsets.push(offset);
        if x.is_none() {
            result.push(Patch::Delete(i));
            offset += 1;
        }
    }

    offset = 0;

    for (i, x) in na.into_iter().enumerate() {
        match x {
            Some(j) => {
                if old[j] != new[i] {
                    result.push(Patch::Update(i));
                }

                if j + offset - delete_offsets[i] != i {
                    result.push(Patch::Move(j, i));
                }
            }
            None => {
                result.push(Patch::Create(i));
                offset += 1;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_changes() {
        let old = vec!["a"];
        let new = vec!["a"];
        let want: Difference = vec![];

        assert_eq!(want, diff(&old, &new));
    }

    #[test]
    fn simple_create() {
        let old = vec!["a"];
        let new = vec!["a", "b"];
        let want = vec![Create(1)];

        assert_eq!(want, diff(&old, &new));
    }

    #[test]
    fn simple_swap() {
        let old = vec!["a", "b"];
        let new = vec!["b", "a"];
        let want = vec![Move(1, 0), Move(0, 1)];

        assert_eq!(want, diff(&old, &new));
    }

    #[test]
    fn swaps() {
        let old = vec!["a", "b", "c"];
        let new = vec!["b", "c", "a"];
        let want = vec![Move(1, 0), Move(2, 1), Move(0, 2)];

        assert_eq!(want, diff(&old, &new));
    }

    #[test]
    fn simple_delete() {
        let old = vec!["a", "b"];
        let new = vec!["a"];
        let want = vec![Delete(1)];

        assert_eq!(want, diff(&old, &new));
    }
}
