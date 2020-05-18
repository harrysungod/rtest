
mod foo;

fn main() {

    let mut _foo = foo::FooConnection::new(String::from(""), String::from(""));
    let data_sources = _foo.get_data().expect("Fetching data failed");

    println!("{:?}", data_sources);
}

