use std::{
    alloc::Global,
    collections::{hash_map::RandomState, HashMap},
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use hash_table::traits::hash_table::HashTable;

use super::{DeterministicHasher, MIN_KEYS};
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng, SeedableRng,
};

mod traits;
use traits::*;

/*
impl<K, V, R> HashTableOperationGenerator<K, V, R>
where
    R: Rng + SeedableRng,
{
    fn from_seed(seed: R::Seed) -> Self {
        Self {
            rng: R::from_seed(seed),
            data: HashMap::new(),
        }
    }
}*/
fn test_hashtable<T, O, K, V, S, R>(
    seed: Option<R::Seed>,
    ops: usize,
    starting_capacity: Option<usize>,
) -> Result<(), OperationFailure<T, O, S, R>>
where
    O: OperationGen<R> + Debug,
    O::Result: Eq,
    K: Hash + Eq + Copy + Debug,
    V: Copy + Eq + Debug,
    <O::Generator as OperationGenerator<R>>::ReferenceImpl: Debug + Default,
    //<O::Generator as OperationGenerator<R>>::Operation::Target: HashTable<K, V, S, Global> + Debug,
    <<<O as OperationGen<R>>::Generator as OperationGenerator<R>>::Operation as Operation>::Target:
        HashTable<K, V, S, Global> + Debug,
    S: BuildHasher + DeterministicHasher<Seed = R::Seed>,
    R: SeedableRng + Rng,
    R::Seed: Clone,
    Standard: Distribution<K>,
    Standard: Distribution<V>,
    Standard: Distribution<R::Seed>,
{
    /*
    let seed: R::Seed = seed.unwrap_or_else(|| rand::thread_rng().gen());
    let starting_capacity = starting_capacity.unwrap_or(50);
    let hash_builder = S::from_seed(seed.clone());
    let mut target =
        T::with_capacity_and_hasher_in(starting_capacity, hash_builder, Global).unwrap();

    let mut gen: O::Generator = O::gen_from_seed(seed.clone());
    let mut op_num = 0;
    for (op, res) in (&mut gen).take(ops) {
        op_num += 1;
        let r = op.apply(&mut target);
        if r != res {
            let mut op_gen = O::gen_from_seed(seed.clone());
            let operations: Vec<_> = (&mut op_gen).take(op_num).collect();
            return Err(OperationFailure {
                seed,
                starting_capacity,
                target,
                op_num,
                operations,
                data: op_gen.data(),
                marker: PhantomData,
            });
        }
    }
    Ok(())
    */
    todo!()
}

#[derive(Debug)]
struct OperationFailure<T, O, S, R>
where
    O: OperationGen<R>,
    S: BuildHasher + DeterministicHasher<Seed = R::Seed>,
    R: SeedableRng + Rng,
    R::Seed: Clone,
    <<O as OperationGen<R>>::Generator as OperationGenerator<R>>::ReferenceImpl: Debug,
{
    pub seed: R::Seed,
    pub starting_capacity: usize,
    pub target: T,
    #[allow(dead_code)]
    pub data: <<O as OperationGen<R>>::Generator as OperationGenerator<R>>::ReferenceImpl,
    #[allow(dead_code)]
    pub operations: Vec<(O, O::Result)>,
    pub op_num: usize,
    marker: PhantomData<S>,
}

impl<T, G, S, R> OperationFailure<T, G, S, R>
where
    G: OperationGenerator<R> + OperationGen<R>,
    G::Operation: Debug,
    G::Result: Debug,
    S: BuildHasher + DeterministicHasher<Seed = R::Seed>,
    R: SeedableRng + Rng,
    <<G as OperationGen<R>>::Generator as OperationGenerator<R>>::ReferenceImpl: Debug,
    R::Seed: Clone,
    //K: Hash + Eq + Debug + Copy,
    Standard: Distribution<R::Seed>,
    //Standard: Distribution<K>,
    //Standard: Distribution<<<G as tester::OperationGenerator<T, R>>::Operation as Trait>::Result>,
{
    fn playback(&self) {
        /*
        println!("running playback");
        let mut table = T::with_capacity_and_hasher_in(
            self.starting_capacity,
            S::from_seed(self.seed.clone()),
            Global,
        )
        .expect("failed alloc");

        let mut gen: OperationGenerator<K, V, R> = OperationGenerator::from_seed(self.seed.clone());
        let lower = self.op_num.saturating_sub(5);
        for _ in 0..lower {
            let (op, r) = gen.next().unwrap();
            assert_eq!(r, op.apply(&mut table));
        }

        for (ind, (op, re)) in gen
            .enumerate()
            .take(std::cmp::min(lower + 5, self.op_num) + 1)
        {
            let r = op.apply(&mut table);
            println!("--------- operation {} ----------", (lower) + ind);
            println!("operation: {:?}", op);
            println!("table_state: {:?}", &self.table);
            println!("expected / actual : {:?}, {:?}", re, r);
        }*/
    }
}
