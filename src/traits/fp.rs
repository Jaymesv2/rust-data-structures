pub trait Functor {
    type Unwrapped;
    type Wrapped<B>: Functor;

    fn map<F, B>(self, f: F) -> Self::Wrapped<B>
    where
        F: FnMut(Self::Unwrapped) -> B;
    /*
    fn replace<F,B: Clone>(self, a: B) -> Self::Wrapped<B> where Self: Sized {
        //let f = move |_: Self::Unwrapped| -> B {a.clone()};
        self.fmap(move |_| a.clone())
    } 
     */
}
/*
pub trait Pointed: Functor {
    fn wrap<T>(t: T) -> Self::Wrapped<T>;
}

 */
/*
Category is a relatively recent addition to the Haskell standard libraries. It generalizes the notion of function composition to general “morphisms”.

The definition of the Category type class (from Control.Category; haddock) is shown below. 
For ease of reading, note that I have used an infix type variable `arr`, in parallel with the infix function type constructor (->). 
∗ This syntax is not part of Haskell 2010. 
The second definition shown is the one used in the standard libraries. 
For the remainder of this document, I will use the infix type constructor `arr` for Category as well as Arrow

class Category a where
  id  :: a `arr` a
  (.) :: (b `arr` c) -> (a `arr` b) -> (a `arr` c)

-- The same thing, with a normal (prefix) type constructor
class Category cat where
  id  :: cat a a
  (.) :: cat b c -> cat a b -> cat a c
*/
// category is the function itself
pub trait Category<Arr> where Arr: Fn(Self::A) -> Self::B {
    type A;
    type B;
    //fn id<A,B>() -> dyn Fn(A) -> B;
    fn id<A>() -> dyn Fn(A,A);

    //fn dot<A,B,C, F1: Fn(A) -> B, F2: Fn(B) -> C>(f1: F1, f2: F2) -> dyn Fn(A) -> C;
    //(.) :: (b `arr` c) -> (a `arr` b) -> (a `arr` c)

    //fn dot(f1: Fn(B,C), f2: Fn(A,B)) -> dyn Fn()
}
/*
newtype Kleisli m a b = Kleisli { runKleisli :: a -> m b }

instance Monad m => Category (Kleisli m) where
  id :: Kleisli m a a
  id = Kleisli return

  (.) :: Kleisli m b c -> Kleisli m a b -> Kleisli m a c
  Kleisli g . Kleisli h = Kleisli (h >=> g)


*/




pub trait Applicative: Functor {
    fn lift_a2<F, B, C>(self, b: Self::Wrapped<B>, f: F) -> Self::Wrapped<C>
    where
        F: FnMut(Self::Unwrapped, B) -> C;
    fn pure<T>(t: T) -> Self::Wrapped<T>;
}

pub trait Monad: Applicative {
    fn bind<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: FnMut(Self::Unwrapped) -> Self::Wrapped<B>;
}

pub trait MonadTrans {
    type Base: Monad;

    fn lift(base: Self::Base) -> Self;
}

pub trait Monoid: Semigroup {
    fn mempty() -> Self;
}

use alloc::vec::Vec;

impl<T> Monoid for Vec<T> {
    fn mempty() -> Vec<T> {
        Vec::new()
    }
}

impl<A> Functor for Vec<A> {
    type Unwrapped = A;
    type Wrapped<B> = Vec<B>;

    fn map<F, B>(self, f: F) -> Self::Wrapped<B>
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

/*
impl<A, const N: usize> Functor for [A; N] {
    type Unwrapped = A;
    type Wrapped<B> = [B; N];
    fn fmap<F, B>(self, f: F) -> Self::Wrapped<B>
        where
            F: FnMut(Self::Unwrapped) -> B {
        let mut iter = self.into_iter().map(f);
        core::array::from_fn(|_| unsafe {iter.next().unwrap_unchecked()})
    }
}
*/

pub trait Semigroup {
    fn append(self, rhs: Self) -> Self;
}

impl<T> Semigroup for Vec<T> {
    fn append(mut self, rhs: Self) -> Self {
        self.extend(rhs.into_iter());
        self
    }
}

trait Zero {
    fn zero() -> Self;
}

use core::ops::Add;

struct Sum<T>(pub T);

impl<T: Add<Output = T>> Semigroup for Sum<T> {
    fn append(self, rhs: Self) -> Self {
        let a = self.0 + rhs.0;
        Sum(a)
    }
}

impl<T: Add<Output = T> + Zero> Monoid for Sum<T> {
    fn mempty() -> Self {
        Sum(T::zero())
    }
}


impl<M: Monad> MonadTrans for IdentityT<M> {
    type Base = M;

    fn lift(base: M) -> Self {
        IdentityT(base)
    }
}

/*
//<M as Functor>::Wrapped<M>
impl<M: Semigroup + Pointed> Semigroup for IdentityT<M> {
    fn append(self, rhs: Self) -> Self {
        M::wrap(self.0.append(rhs.0))
    }
}
*/
struct IdentityT<M>(M);

impl<M: Functor> Functor for IdentityT<M> {
    type Unwrapped = M::Unwrapped;
    type Wrapped<A> = IdentityT<M::Wrapped<A>>;

    fn map<F, B>(self, f: F) -> Self::Wrapped<B>
    where
        F: FnMut(M::Unwrapped) -> B,
    {
        IdentityT(self.0.map(f))
        //Option::None
    }
}
/*
impl<M: Pointed> Pointed for IdentityT<M> {
    fn wrap<T>(t: T) -> IdentityT<M::Wrapped<T>> {
        IdentityT(M::wrap(t))
    }
}
 */

impl<M: Applicative> Applicative for IdentityT<M> {
    fn lift_a2<F, B, C>(self, b: Self::Wrapped<B>, f: F) -> Self::Wrapped<C>
    where
        F: FnMut(Self::Unwrapped, B) -> C,
    {
        IdentityT(self.0.lift_a2(b.0, f))
    }

    fn pure<T>(t: T) -> Self::Wrapped<T> {
        //<M as Functor>::Wrapped<<M as Functor>::Unwrapped>
        IdentityT(M::pure(t))
    }
}

impl<M: Monad> Monad for IdentityT<M> {
    fn bind<B, F>(self, mut f: F) -> Self::Wrapped<B>
    where
        F: FnMut(Self::Unwrapped) -> Self::Wrapped<B>,
    {
        IdentityT(self.0.bind(|x| f(x).0))
    }
}

impl<A> Functor for Option<A> {
    type Unwrapped = A;
    type Wrapped<B> = Option<B>;

    fn map<F: FnMut(A) -> B, B>(self, mut f: F) -> Option<B> {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }
}

impl<A, E> Functor for Result<A, E> {
    type Unwrapped = A;
    type Wrapped<B> = Result<B, E>;

    fn map<F: FnMut(A) -> B, B>(self, mut f: F) -> Result<B, E> {
        match self {
            Ok(x) => Ok(f(x)),
            Err(e) => Err(e),
        }
    }
}
/*
impl<A> Pointed for Option<A> {
    fn wrap<T>(t: T) -> Self::Wrapped<T> {
        Some(t)
    }
}

impl<A, E> Pointed for Result<A, E> {
    fn wrap<T>(t: T) -> Self::Wrapped<T> {
        Ok(t)
    }
}
 */

impl<A> Applicative for Option<A> {
    fn lift_a2<F, B, C>(self, b: Self::Wrapped<B>, mut f: F) -> Self::Wrapped<C>
    where
        F: FnMut(Self::Unwrapped, B) -> C,
    {
        let a = self?;
        let b = b?;
        Some(f(a, b))
    }
    fn pure<T>(t: T) -> Self::Wrapped<T> {
        Some(t)
    }
}

impl<A, E> Applicative for Result<A, E> {
    fn lift_a2<F, B, C>(self, b: Self::Wrapped<B>, mut f: F) -> Self::Wrapped<C>
    where
        F: FnMut(Self::Unwrapped, B) -> C,
    {
        let a = self?;
        let b = b?;
        Ok(f(a, b))
    }
    fn pure<T>(t: T) -> Self::Wrapped<T> {
        Ok(t)
    }
}

impl<A> Monad for Option<A> {
    fn bind<B, F>(self, f: F) -> Option<B>
    where
        F: FnMut(A) -> Option<B>,
    {
        self.and_then(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn option_functor() {
        let x2 = |i| i * 2;
        let i = Option::<i32>::pure(32).map(x2);
        assert_eq!(i, Some(64));
    }
    #[test]
    fn vec_functor() {
        let list: Vec<u32> = (0..100).collect();
        let add_5 = |x| x+5;
        let list2 = list.map(add_5);
        assert_eq!((5..105).collect::<Vec<u32>>(), list2);
    }
}
