#![feature(type_alias_impl_trait)]

fn main() {
    let y = 42;
    let x = wrong_generic(&y);
    let z: i32 = x;
    //~^ ERROR non-defining opaque type use
}

type WrongGeneric<T> = impl 'static;
//~^ ERROR: at least one trait must be specified
//~| ERROR could not find defining uses

fn wrong_generic<T>(t: T) -> WrongGeneric<T> {
    t
    //~^ ERROR the parameter type `T` may not live long enough
}
