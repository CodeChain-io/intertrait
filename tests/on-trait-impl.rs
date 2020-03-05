use intertrait::CastTo;
use intertrait::*;

struct Data;

trait Source: CastFrom {}

trait Greet {
    fn greet(&self);
}

#[cast_to]
impl Greet for Data {
    fn greet(&self) {
        println!("Hello");
    }
}

impl Source for Data {}

#[test]
fn test_cast_to_on_trait_impl() {
    let data = Data;
    let source: &dyn Source = &data;
    let greet = source.ref_to::<dyn Greet>();
    greet.unwrap().greet();
}
