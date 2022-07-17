use ebur128::EbuR128;
use log::debug;

use crate::replay_gain::{album_rg, track_rg, ReplayGain};

mod audio;
mod options;
mod replay_gain;

fn main() {
    let opts = options::parse_arguments();
    debug!("{:#?}", opts);

    let tracks: Vec<(ReplayGain, EbuR128)> = opts
        .files
        .iter()
        .map(|x| track_rg(x, opts.pre_gain).unwrap())
        .collect();

    let album = if opts.do_album {
        Some(album_rg(&tracks, opts.pre_gain).unwrap().clipper(
            opts.max_true_peak_level,
            opts.warn_clip,
            opts.clip_prevention,
        ))
    } else {
        None
    };

    let tracks = tracks.iter().map(|(rg, _)| {
        rg.clipper(
            opts.max_true_peak_level,
            opts.warn_clip,
            opts.clip_prevention,
        )
    });
}
