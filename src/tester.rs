use crate::HashTableImpl;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

use rand::distributions::{Distribution, Standard};
use rand::{Fill, SeedableRng};
use rand::rngs::StdRng;

use std::collections::hash_map::{RandomState, DefaultHasher};
/* 

unsafe fn randomstate_from_seed(seed: [u8;32]) -> RandomState {
    
    //RandomState
}*/

const MIN_KEYS: usize = 1;
#[test]
fn run_test() {
    //let seed = Some([156, 193, 116, 26, 251, 136, 38, 106, 25, 11, 3, 103, 121, 63, 45, 216, 137, 198, 232, 43, 43, 113, 252, 174, 158, 83, 147, 231, 40, 230, 246, 215]);
    let seed = None;
    //let seed = Some([106, 115, 125, 98, 35, 103, 155, 128, 225, 226, 14, 60, 5, 203, 73, 72, 97, 49, 214, 19, 115, 143, 170, 192, 81, 70, 103, 102, 196, 89, 69, 106]);
    for _ in 0..1000 {
        //if let Err(report) = test_hashtable::<crate::HashTable<u8, i64, RandomState>, _, _, RandomState, StdRng>(seed, 4, Some(3)) {
        if let Err(report) = test_hashtable::<crate::HashTable<u8, i64, RandomState>, _, _, RandomState, StdRng>(seed, 1000, None) {
            println!("{:?}", report);
            report.playback();
            panic!("died");
        }
    }
    
}

trait DeterministicHasher: BuildHasher {
    type Seed;
    fn from_seed(seed: Self::Seed) -> Self;
}

impl DeterministicHasher for RandomState {
    type Seed = [u8;32];
    // I'm trying to make the test_hashtable function deterministic but randomstate is annoying :/
    fn from_seed(seed: Self::Seed) -> Self {
        // RandomState is `RandomState {k1: u64, k2: u64}` so its equal size to (u64, u64). transmuting should be fine.
        unsafe {
            use std::mem::{transmute, size_of};
            #[allow(dead_code)]
            const SIZE_CHECK: u8 = (size_of::<(u64,u64)>() == size_of::<RandomState>()) as u8 -1;
            let a: [u64;4] = transmute(seed);
            transmute((a[0]^a[1], a[2]^a[3]))
        }
    }
}

fn test_hashtable<H, K, V, S, R>(seed: Option<R::Seed>, ops: usize, starting_capacity: Option<usize>) -> Result<(), HashTableFailure<H, K, V, S, R>>
where
    H: HashTableImpl<K, V, S> + Default + Debug,
    K: Hash + Eq + Copy + Eq + Debug,
    V: Copy + Eq + Debug,
    S: BuildHasher + DeterministicHasher<Seed = R::Seed> + Default,
    R: SeedableRng + Rng,
    R::Seed: Clone,
    Standard: Distribution<K>,
    Standard: Distribution<V>,
    Standard: Distribution<R::Seed>,
{
    let seed: R::Seed = seed.unwrap_or_else(|| rand::thread_rng().gen());
    let starting_capacity = starting_capacity.unwrap_or(50);
    let hash_builder = S::from_seed(seed.clone());
    let mut table = H::with_capacity_and_hasher(starting_capacity, hash_builder);

    let mut gen: OperationGenerator<K,V,R> = OperationGenerator::from_seed(seed.clone());
    let mut op_num = 0;
    for (op, res) in (&mut gen).take(ops) {
        op_num += 1;
        let r = op.apply(&mut table);
        if r != res {
            let mut op_gen = OperationGenerator::<K,V,R>::from_seed(seed.clone());
            let operations: Vec<_> = (&mut op_gen).take(op_num).collect();
            return Err(HashTableFailure {
                seed,
                starting_capacity,
                table,
                op_num,
                operations,
                data: op_gen.data,
                marker: PhantomData,
            })
        }
    }

    Ok(())
}

use std::iter::*;

#[derive(Debug)]
struct HashTableFailure<H, K, V, S, R>
where
    H: HashTableImpl<K, V, S> + Default + Debug,
    S: BuildHasher + Default,
    R: SeedableRng + Rng,
    R::Seed: Clone,
    K: Hash + Eq + Debug,
    V: Debug,
{
    pub seed: R::Seed,
    pub starting_capacity: usize,
    pub table: H,
    pub data: HashMap<K, V>,
    pub operations: Vec<(Operation<K, V>, Option<V>)>,
    pub op_num: usize,
    marker: PhantomData<S>,
}

impl<H, K, V, S, R> HashTableFailure<H, K, V, S, R>
where
    H: HashTableImpl<K, V, S> + Default + Debug,
    S: BuildHasher + DeterministicHasher<Seed = R::Seed> +Default,
    R: SeedableRng + Rng,
    R::Seed: Clone,
    K: Hash + Eq + Debug + Copy,
    V: Debug + Copy + Eq,
    Standard: Distribution<R::Seed>,
    Standard: Distribution<K>,
    Standard: Distribution<V>,
{
    fn playback(&self) {
        println!("running playback");
        let mut table = H::with_capacity_and_hasher(self.starting_capacity, S::from_seed(self.seed.clone()));

        let mut gen: OperationGenerator<K,V,R> = OperationGenerator::from_seed(self.seed.clone());
        let lower = self.op_num.saturating_sub(5);
        for _ in 0..lower {
            let (op, r) = gen.next().unwrap();
            assert_eq!(r, op.apply(&mut table));
        }

        for (ind, (op, re)) in gen.enumerate().take(std::cmp::min(lower+5, self.op_num)+1) {
            let r = op.apply(&mut table);
            println!("--------- operation {} ----------", (lower)+ind);
            println!("operation: {:?}", op);
            println!("table_state: {:?}", &self.table);
            println!("expected / actual : {:?}, {:?}", re,r);
        }
        
    }
}



struct OperationGenerator<K, V, R = StdRng>
where
    R: Rng + SeedableRng,
{
    rng: R,
    data: HashMap<K, V>,
}

impl<K, V, R> OperationGenerator<K, V, R>
where
    R: Rng + SeedableRng,
{
    fn from_seed(seed: R::Seed) -> Self {
        Self {
            rng: R::from_seed(seed),
            data: HashMap::new(),
        }
    }
}


impl<K, V, R> Iterator for OperationGenerator<K, V,R>
where
    K: Copy + Hash + Eq,
    V: Copy,
    R: SeedableRng + Rng,
    Standard: Distribution<K>,
    Standard: Distribution<V>,
{
    type Item = (Operation<K, V>, Option<V>);

    fn next(&mut self) -> Option<Self::Item> {
        //  this prevents insert operations if there arent enough keys
        let range = if self.data.len() > MIN_KEYS {
            0..std::mem::variant_count::<Operation<K, V>>()
        } else {
            0..1
        };

        match self.rng.gen_range(range) {
            0 => {
                let key: K = self.rng.gen();
                let val: V = self.rng.gen();
                let res: Option<V> = self.data.insert(key, val);
                Some((Operation::Insert(key, val), res))
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

                Some((Operation::Get(key), res.copied()))
            }
            2 => {
                let key: K = {
                    let ind = self.rng.gen_range(0..self.data.len());
                    *self.data.keys().nth(ind).unwrap()
                };
                Some((Operation::Remove(key), self.data.remove(&key)))
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation<K, V> {
    Insert(K, V),
    Remove(K),
    Get(K),
}

use rand::Rng;

impl<K, V> Operation<K, V>
where
    K: Hash + Eq + Copy + Eq + Debug,
    V: Copy + Eq + Debug,
{
    fn apply<S: BuildHasher + Default, H: HashTableImpl<K, V, S>>(
        &self,
        table: &mut H,
    ) -> Option<V> {
        match self {
            Self::Insert(key, value) => table.insert(*key, *value),
            Self::Get(key) => table.get(key).copied(),
            Self::Remove(key) => table.remove(key),
        }
    }
}
