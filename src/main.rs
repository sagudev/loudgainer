use std::fs::File;

use ebur128::EbuR128;

use crate::replay_gain::{track_rg, ReplayGain};

mod audio;
mod options;
mod replay_gain;

fn main() {
    let opts = options::parse_arguments();

    let scans: Vec<(ReplayGain, EbuR128)> = opts
        .files
        .iter()
        .map(|x| track_rg(x, opts.pre_gain).unwrap())
        .collect();

    println!("{:#?}", opts);
}
