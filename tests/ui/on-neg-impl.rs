use intertrait::*;

struct Data;

#[cast_to]
impl !Sync for Data {}

fn main() {
    let data = Data;
    let _: &dyn Sync = &data;
}
