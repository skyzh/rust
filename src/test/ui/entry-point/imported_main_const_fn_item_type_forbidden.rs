#![feature(imported_main)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
//~^^^ ERROR `main` function not found in crate
pub mod foo {
    type MainFn = impl Fn();

    fn bar() {}
    pub const BAR: MainFn = bar;
}

use foo::BAR as main;
