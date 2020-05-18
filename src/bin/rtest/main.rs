
mod foo;

fn main() {

    let mut _foo = foo::FooConnection::new("", "");
    let data_sources = _foo.get_data().expect("Fetching data failed");

    println!("{:?}", data_sources);
}

