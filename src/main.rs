mod options;

fn main() {
    let opts = options::parse_arguments();
    println!("{:#?}", opts);
}
