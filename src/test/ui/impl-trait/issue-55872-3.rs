// edition:2018
// ignore-compare-mode-chalk

#![feature(type_alias_impl_trait)]

pub trait Bar {
    type E: Copy;

    fn foo<T>() -> Self::E;
}

impl<S> Bar for S {
    type E = impl std::marker::Copy;
    //~^ ERROR could not find defining uses
    fn foo<T>() -> Self::E {
        async {}
        //~^ ERROR the trait bound `impl Future<Output = [async output]>: Copy` is not satisfied [E0277]
    }
}

fn main() {}
