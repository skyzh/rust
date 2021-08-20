#![feature(type_alias_impl_trait)]

use std::fmt::Debug;

fn main() {}

// test that unused generic parameters are ok
type Two<T, U> = impl Debug;
//~^ ERROR `T` doesn't implement `Debug`

fn one<T: Debug>(t: T) -> Two<T, T> {
    t
    //~^ ERROR non-defining opaque type use in defining scope
}

fn two<T: Debug, U>(t: T, _: U) -> Two<T, U> {
    t
}
