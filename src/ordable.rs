use std::collections::BTreeSet;

/* # dedup */

/// removes consecutive equal elements
pub struct DedupOrd<I>
where
    I: Iterator,
{
    iterator: I,
    seen: BTreeSet<I::Item>,
}

impl<I> Iterator for DedupOrd<I>
where
    I: Iterator,
    I::Item: Ord + Clone,
{
    type Item = I::Item;

    #[allow(clippy::while_let_on_iterator, reason = "seems more readable here")]
    fn next(&mut self) -> Option<I::Item> {
        while let Some(item) = self.iterator.next() {
            if !self.seen.contains(&item) {
                self.seen.insert(item.clone());
                return Some(item);
            }
        }
        None
    }
}

/// provides the `dedup` method on `Iterator`s
pub trait DedupOrdAdapter: Iterator {
    fn dedup_ord(self) -> DedupOrd<Self>
    where
        Self: Sized,
    {
        DedupOrd {
            seen: BTreeSet::new(),
            iterator: self,
        }
    }
}

impl<I> DedupOrdAdapter for I where I: Iterator {}

/* # dedup by */

/// removes consecutive elements, whose equality is asserted by provided function
pub struct DedupOrdBy<I, F>
where
    I: Iterator,
{
    iterator: I,
    seen: BTreeSet<I::Item>,
    equivalence: F,
}

impl<I, F> Iterator for DedupOrdBy<I, F>
where
    I: Iterator,
    I::Item: Ord + Clone,
    F: Fn(&I::Item, &I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        while let Some(item) = self.iterator.next() {
            if !self.seen.iter().any(|old| (self.equivalence)(old, &item)) {
                self.seen.insert(item.clone());
                return Some(item);
            }
        }
        None
    }
}

/// provides the `dedup_by` method on `Iterator`s
pub trait DedupOrdByAdapter<F>: Iterator {
    fn dedup_ord_by(self, equivalence: F) -> DedupOrdBy<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Item, &Self::Item) -> bool,
    {
        DedupOrdBy {
            seen: BTreeSet::new(),
            iterator: self,
            equivalence,
        }
    }
}

impl<I, F> DedupOrdByAdapter<F> for I where I: Iterator {}

/* # dedup by key */

/// removes consecutive elements, which give equal outputs from provided function
pub struct DedupOrdByKey<I, F, K>
where
    I: Iterator,
    F: Fn(&I::Item) -> K,
{
    iterator: I,
    seen: BTreeSet<K>,
    function: F,
}

impl<I, F, K> Iterator for DedupOrdByKey<I, F, K>
where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: Ord,
{
    type Item = I::Item;

    #[allow(clippy::while_let_on_iterator, reason = "seems more readable here")]
    fn next(&mut self) -> Option<I::Item> {
        while let Some(item) = self.iterator.next() {
            let key = (self.function)(&item);
            if !self.seen.contains(&key) {
                self.seen.insert(key);
                return Some(item);
            }
        }
        None
    }
}

/// Provides the `dedup_by_key` method on `Iterator`s.
pub trait DedupOrdByKeyAdapter<F, K>: Iterator {
    fn dedup_ord_by_key(self, function: F) -> DedupOrdByKey<Self, F, K>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> K,
    {
        DedupOrdByKey {
            seen: BTreeSet::new(),
            iterator: self,
            function,
        }
    }
}

impl<I, F, K> DedupOrdByKeyAdapter<F, K> for I where I: Iterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicate_empty_iterator() {
        let og = Vec::<u8>::new();
        let dp = og.iter().dedup_ord().count();
        assert_eq!(dp, 0);
    }

    #[test]
    fn remove_duplicate_character() {
        let og = "ssporrro podwojjonnyych litterr";
        let dp = og.chars().dedup_ord().collect::<String>();
        assert_eq!(&dp, "spor dwjnychlite");
    }

    #[test]
    fn remove_duplicate_number() {
        let og: [i32; 7] = [10, 20, 20, 21, 30, 30, 20];
        let dp = og.into_iter().dedup_ord().collect::<Vec<_>>();
        let re: [i32; 4] = [10, 20, 21, 30];
        assert_eq!(dp, re);
    }

    #[test]
    fn remove_duplicate_whitespace() {
        let og = "ttu    teżż  czasem   jakkaś litterka     dwa  rrazy";
        let dp = og
            .chars()
            .dedup_ord_by(|a, b| a.is_whitespace() && b.is_whitespace())
            .collect::<String>();
        assert_eq!(&dp, "ttu teżżczasemjakkaślitterkadwarrazy");
    }

    #[test]
    fn deduplicate_by_equality() {
        let og = "ttu    teżż  czasem   jakkaś litterka     dwa  rrazy";
        let dp = og.chars().dedup_ord_by(|&a, &b| a == b).collect::<String>();
        assert_eq!(&dp, "tu eżczasmjkślirdwy");
    }

    #[test]
    fn dedup_by_key() {
        #[derive(Debug, PartialEq)]
        struct Test {
            id: u8,
            other: Vec<u8>,
        }
        let og = [
            Test {
                id: 0,
                other: vec![0, 1, 2],
            },
            Test {
                id: 0,
                other: vec![0, 1, 2, 3],
            },
        ];
        let dp = og
            .into_iter()
            .dedup_ord_by_key(|test| test.id)
            .collect::<Vec<_>>();
        assert_eq!(
            dp,
            [Test {
                id: 0,
                other: vec![0, 1, 2],
            },]
        );
    }

    #[test]
    fn deduplicate_by_always_same() {
        let t = "abdefghijklmopqrstuvwxyz";
        let v = t.chars().dedup_ord_by(|_, _| true).collect::<String>();
        assert_eq!(&v, "a");
    }

    #[test]
    fn deduplicate_by_key_always_same() {
        let t = "abdefghijklmopqrstuvwxyz";
        let v = t.chars().dedup_ord_by_key(|_| 0_u8).collect::<String>();
        assert_eq!(&v, "a");
    }
}
