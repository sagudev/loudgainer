mod audio;
mod options;
mod replay_gain;

fn main() {
    let opts = options::parse_arguments();

    println!("{:#?}", opts);
}
