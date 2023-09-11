/* # dedup */

/// removes consecutive equal elements
#[derive(Debug, Clone)]
pub struct Dedup<I>
where
    I: Iterator,
{
    iterator: I,
    current: Option<I::Item>,
}

impl<I> Iterator for Dedup<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let current = self.current.take()?;
        let self_current = &mut self.current;
        Some(
            self.iterator
                .try_fold(current, |acc, next| match acc == next {
                    true => Ok(next),
                    false => {
                        *self_current = Some(next);
                        Err(acc)
                    }
                })
                .unwrap_or_else(|x| x),
        )
    }
}

/// provides the `dedup` method on `Iterator`s
pub trait DedupAdapter: Iterator {
    fn dedup(mut self) -> Dedup<Self>
    where
        Self: Sized,
    {
        Dedup {
            current: self.next(),
            iterator: self,
        }
    }
}

impl<I> DedupAdapter for I where I: Iterator {}

/* # dedup by */

/// removes consecutive elements, whose equality is asserted by provided function
#[derive(Debug, Clone)]
pub struct DedupBy<I, F>
where
    I: Iterator,
{
    iterator: I,
    current: Option<I::Item>,
    equivalence: F,
}

impl<I, F> Iterator for DedupBy<I, F>
where
    I: Iterator,
    F: Fn(&I::Item, &I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let current = self.current.take()?;
        let self_current = &mut self.current;
        Some(
            self.iterator
                .try_fold(current, |acc, next| match (self.equivalence)(&acc, &next) {
                    true => Ok(next),
                    false => {
                        *self_current = Some(next);
                        Err(acc)
                    }
                })
                .unwrap_or_else(|x| x),
        )
    }
}

/// provides the `dedup_by` method on `Iterator`s
pub trait DedupByAdapter<F>: Iterator {
    fn dedup_by(mut self, equivalence: F) -> DedupBy<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Item, &Self::Item) -> bool,
    {
        DedupBy {
            current: self.next(),
            iterator: self,
            equivalence,
        }
    }
}

impl<I, F> DedupByAdapter<F> for I where I: Iterator {}

/* # dedup by key */

/// removes consecutive elements, which give equal outputs from provided function
#[derive(Debug, Clone)]
pub struct DedupByKey<I, F, K>
where
    I: Iterator,
    F: Fn(&I::Item) -> K,
{
    iterator: I,
    current: Option<I::Item>,
    function: F,
}

impl<I, F, K> Iterator for DedupByKey<I, F, K>
where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let current = self.current.take()?;
        let self_current = &mut self.current;
        Some(
            self.iterator
                .try_fold(current, |acc, next| {
                    match (self.function)(&acc) == (self.function)(&next) {
                        true => Ok(next),
                        false => {
                            *self_current = Some(next);
                            Err(acc)
                        }
                    }
                })
                .unwrap_or_else(|x| x),
        )
    }
}

/// Provides the `dedup_by_key` method on `Iterator`s.
pub trait DedupByKeyAdapter<F, K>: Iterator {
    fn dedup_by_key(mut self, function: F) -> DedupByKey<Self, F, K>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> K,
    {
        DedupByKey {
            current: self.next(),
            iterator: self,
            function,
        }
    }
}

impl<I, F, K> DedupByKeyAdapter<F, K> for I where I: Iterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicate_empty_iterator() {
        let og = Vec::<u8>::new();
        let dp = og.iter().dedup().count();
        assert_eq!(dp, 0);
    }

    #[test]
    fn remove_duplicate_character() {
        let og = "ssporrro podwojjonnyych litterr";
        let dp = og.chars().dedup().collect::<String>();
        assert_eq!(&dp, "sporo podwojonych liter");
    }

    #[test]
    fn remove_duplicate_number() {
        let og: [i32; 7] = [10, 20, 20, 21, 30, 30, 20];
        let dp = og.into_iter().dedup().collect::<Vec<_>>();
        let re: [i32; 5] = [10, 20, 21, 30, 20];
        assert_eq!(dp, re);
    }

    #[test]
    fn remove_duplicate_whitespace() {
        let og = "ttu    teżż  czasem   jakkaś litterka     dwa  rrazy";
        let dp = og
            .chars()
            .dedup_by(|a, b| a.is_whitespace() && b.is_whitespace())
            .collect::<String>();
        assert_eq!(&dp, "ttu teżż czasem jakkaś litterka dwa rrazy");
    }

    #[test]
    fn deduplicate_by_equality() {
        let og = "ttu    teżż  czasem   jakkaś litterka     dwa  rrazy";
        let dp = og.chars().dedup_by(|&a, &b| a == b).collect::<String>();
        assert_eq!(&dp, "tu też czasem jakaś literka dwa razy");
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
            .dedup_by_key(|test| test.id)
            .collect::<Vec<_>>();
        assert_eq!(
            dp,
            [Test {
                id: 0,
                other: vec![0, 1, 2, 3],
            },]
        );
    }

    #[test]
    fn deduplicate_by_always_same() {
        let t = "abdefghijklmopqrstuvwxyz";
        let v = t.chars().dedup_by(|_, _| true).collect::<String>();
        assert_eq!(&v, "z");
    }

    #[test]
    fn deduplicate_by_key_always_same() {
        let t = "abdefghijklmopqrstuvwxyz";
        let v = t.chars().dedup_by_key(|_| 0_u8).collect::<String>();
        assert_eq!(&v, "z");
    }
}
