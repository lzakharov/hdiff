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
    nc: usize,   // number of copies of this line in new file
    oc: usize,   // number of copies of this line in old file
    olno: usize, // line number in the old file
}

type Array = Vec<Entry>;

type Entry = Option<usize>;

//
// Pass 1 comprises the following:
// (a) each line i of file N is read in sequence;
// (b) a symbol table entry for each line i is
//     created if it does not already exist;
// (c) NC for the line's symbol table entry is incremented; and
// (d) NA[i] is set to point to the symbol table entry of line i.
//
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

//
// Pass 2 is identical to pass 1 except that it acts on
// file O, array OA, and counter OC, and OLNO for the
// symbol table entry is set to the line's number.
//
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

//
// In pass 3 we use observation #1 and process only
// those lines having NC = OC = 1. Since each represents
// (we assume) the same unmodified line, for each we
// replace the symbol table pointers in NA and OA by
// the number of the line in the other file. For example,
// if NA[i] corresponds to such a line, we look NA[i] up
// in the symbol table and set NA[i] to OLNO and
// OA[OLNO] to i. In pass 3 we also "find" unique
// virtual lines immediately before the first and immediately
// after the last lines of the files.
//
// Observation #1: A line that occurs once and only once in each
// file must be the same line (unchanged but possibly moved).
//
fn pass3<T: Eq + Hash>(new: &[T], table: Table<&T>, oa: &mut Array, na: &mut Array) {
    for i in 0..na.len() {
        let tx = &table[&new[i]];
        if tx.nc == 1 && tx.nc == tx.oc {
            na[i] = Some(tx.olno);
            oa[tx.olno] = Some(i);
        }
    }
}

//
// In pass 4, we apply observation #2 and process each
// line in NA in ascending order: If NA[i] points to
// OA[j] and NA[i + 1] and OA[j + 1] contain identical
// symbol table entry pointers, then OA[j + 1] is set to
// line i + 1 and NA[i + 1] is set to line j + 1.
//
// Observation #2: If in each file immediately adjacent to a "found"
// line pair there are lines identical to each other, these
// lines must be the same line.
//
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

//
// In pass 5, we also apply observation #2 and process
// each entry in descending order: if NA[i] points to
// OA[j] and NA[i - 1] and OA[j - 1] contain identical
// symbol table pointers, then NA[i - 1] is replaced by
// j - 1 and OA[j - 1] is replaced by i - 1.
//
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

//
// Array NA now contains the information needed to
// list (or encode) the differences: If NA[i] points to a
// symbol table entry, then we assume that line i is an
// insert, and we can flag it as new text. If it points to
// OA[j], but NA[i + 1] does not point to OA[j + 1],
// then line i is at the boundary of a deletion or block
// move and can be flagged as such. In the final pass, the
// file is output with its changes described in a form
// appropriate to a particular application environment.
//
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

                if j + offset - delete_offsets[j] != i {
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
    fn b_ab() {
        let old = vec!["b"];
        let new = vec!["a", "b"];
        let want = vec![Create(0)];

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

    #[test]
    fn a_mass_much_writing() {
        let old = vec!["a", "mass", "of", "latin", "words",
                       "falls", "upon", "the", "relevant", "facts",
                       "like", "soft", "snow", ",", "covering",
                       "up", "the", "details", "."];
        let new = vec!["much", "writing", "is", "like", "snow",
                       ",", "a", "mass", "of", "long",
                       "words", "and", "phrases", "falls", "upon",
                       "the", "relevant", "facts", "covering", "up",
                       "the", "details", "."];
        let want = vec![Delete(3), Delete(11), Create(0), Create(1), Create(2),
                        Move(10, 3), Move(12, 4), Move(13, 5), Move(0, 6), Move(1, 7),
                        Move(2, 8), Create(9), Move(4, 10), Create(11), Create(12),
                        Move(5, 13), Move(6, 14), Move(7, 15), Move(8, 16), Move(9, 17)];

        assert_eq!(want, diff(&old, &new));
    }

    #[test]
    fn much_writing_a_mass() {
        let old = vec!["much", "writing", "is", "like", "snow",
                       ",", "a", "mass", "of", "long",
                       "words", "and", "phrases", "falls", "upon",
                       "the", "relevant", "facts", "covering", "up",
                       "the", "details", "."];
        let new = vec!["a", "mass", "of", "latin", "words",
                       "falls", "upon", "the", "relevant", "facts",
                       "like", "soft", "snow", ",", "covering",
                       "up", "the", "details", "."];
        let want = vec![Delete(0), Delete(1), Delete(2), Delete(9), Delete(11),
                        Delete(12), Move(6, 0), Move(7, 1), Move(8, 2), Create(3),
                        Move(10, 4), Move(13, 5), Move(14, 6), Move(15, 7), Move(16, 8),
                        Move(17, 9), Move(3, 10), Create(11), Move(4, 12), Move(5, 13)];

        assert_eq!(want, diff(&old, &new));
    }
}
