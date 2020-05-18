use regex;

fn test1() {

    let mut _foo = foo::FooConnection::new(String::from(""), String::from(""));
    let dses = _foo.get_data().expect("Fetching data failed");

    println!("{:?}", dses);
}
