use rand::{
    distributions::{Distribution, Standard},
    Rng, SeedableRng,
};

/// deterministically applies an operation to a T returning the result
pub trait Operation {
    type Result: Sized;
    type Target;
    fn apply(&self, target: &mut Self::Target) -> Self::Result;
}

pub trait OperationGen<R: Rng + SeedableRng>: Operation + Sized {
    type Generator: OperationGenerator<R, Operation = Self, Item = (Self, Self::Result)>;
    fn gen_from_seed(seed: R::Seed) -> Self::Generator {
        Self::Generator::from_seed(seed)
    }
}

// deterministically generates a sequence of valid operations
pub trait OperationGenerator<R>:
    Iterator<Item = (Self::Operation, <Self::Operation as Operation>::Result)> + Sized
where
    R: Rng + SeedableRng,
{
    type Operation: Operation;
    type ReferenceImpl;
    fn from_seed(seed: R::Seed) -> Self;
    fn data(self) -> Self::ReferenceImpl;
}

// extends operation generator so that it can be created from a random seed if the seed can be randomly generated.
pub trait RandomOperationGen<R: SeedableRng + Rng>: OperationGen<R>
where
    Standard: Distribution<R::Seed>,
{
    fn gen() -> Self::Generator {
        let seed: R::Seed = rand::thread_rng().gen();
        Self::Generator::from_seed(seed)
    }
}

impl<R, T: OperationGen<R>> RandomOperationGen<R> for T
where
    R: SeedableRng + Rng,
    Standard: Distribution<R::Seed>,
{
}

mod impls {
    use super::*;
    use crate::MIN_KEYS;
    use core::fmt::Debug;
    use core::hash::Hash;
    use hash_table::traits::hash_table::HashTable;
    use rand::rngs::StdRng;
    use std::alloc::Global;
    use std::collections::hash_map::RandomState;
    use std::collections::HashMap;
    use std::marker::PhantomData;

    #[derive(Clone, Copy, Debug)]
    enum HashTableOperation<I, K, V>
    where
        I:,
    {
        Insert(K, V),
        Remove(K),
        Get(K),
        #[allow(dead_code)]
        Marker(PhantomData<I>),
    }

    impl<T, K, V> Operation for HashTableOperation<T, K, V>
    where
        T: HashTable<K, V, RandomState, Global>,
        K: Hash + Eq + Copy + Eq + Debug,
        V: Copy + Eq + Debug,
        //T: HashTable<K, V, RandomState, Global>,
        Standard: Distribution<K>,
        Standard: Distribution<V>,
    {
        type Result = Option<V>;
        type Target = T;

        fn apply(&self, table: &mut Self::Target) -> Self::Result {
            match self {
                Self::Insert(key, value) => table.insert(*key, *value).expect("failed alloc"),
                Self::Get(key) => table.get(key).copied(),
                Self::Remove(key) => table.remove(key),
                _ => None,
            }
        }
    }

    struct HashTableOperationGenerator<K, V, I, R = StdRng>
    where
        R: Rng + SeedableRng,
        I: HashTable<K, V, RandomState, Global>,
    {
        rng: R,
        data: HashMap<K, V>,
        marker: PhantomData<I>,
    }

    impl<K, V, I, R> OperationGenerator<R> for HashTableOperationGenerator<K, V, I, R>
    where
        R: Rng + SeedableRng,
        I: HashTable<K, V, RandomState, Global>,
        K: Copy + Hash + Eq + Debug,
        V: Copy + Eq + Debug,
        Standard: Distribution<K>,
        Standard: Distribution<V>,
    {
        type Operation = HashTableOperation<I, K, V>;
        type ReferenceImpl = HashMap<K, V>;
        fn from_seed(seed: R::Seed) -> Self {
            Self {
                rng: R::from_seed(seed),
                data: HashMap::new(),
                marker: PhantomData,
            }
        }
        fn data(self) -> Self::ReferenceImpl {
            self.data
        }
    }

    impl<K, V, I, R> Iterator for HashTableOperationGenerator<K, V, I, R>
    where
        I: HashTable<K, V, RandomState, Global>,
        K: Copy + Hash + Eq,
        V: Copy,
        R: SeedableRng + Rng,
        Standard: Distribution<K>,
        Standard: Distribution<V>,
    {
        type Item = (HashTableOperation<I, K, V>, Option<V>);

        fn next(&mut self) -> Option<Self::Item> {
            //  this prevents insert operations if there arent enough keys
            let range = if self.data.len() > MIN_KEYS {
                0..(std::mem::variant_count::<HashTableOperation<I, K, V>>() - 1)
            } else {
                0..1
            };

            match self.rng.gen_range(range) {
                0 => {
                    let key: K = self.rng.gen();
                    let val: V = self.rng.gen();
                    let res: Option<V> = self.data.insert(key, val);
                    Some((HashTableOperation::Insert(key, val), res))
                }
                1 => {
                    let get_existing = self.rng.gen_bool(0.75);
                    let key: K = if get_existing {
                        let ind = self.rng.gen_range(0..self.data.len());
                        *self.data.keys().nth(ind).unwrap()
                    } else {
                        self.rng.gen()
                    };

                    let res = self.data.get(&key);

                    Some((HashTableOperation::Get(key), res.copied()))
                }
                2 => {
                    let key: K = {
                        let ind = self.rng.gen_range(0..self.data.len());
                        *self.data.keys().nth(ind).unwrap()
                    };
                    Some((HashTableOperation::Remove(key), self.data.remove(&key)))
                }
                _ => unreachable!(),
            }
        }
    }
}
