use ebur128::EbuR128;
use log::debug;

use crate::replay_gain::{album_rg, track_rg, ReplayGain};

mod audio;
mod options;
mod replay_gain;
mod tagger;

fn main() {
    let opts = options::parse_arguments();
    debug!("{:#?}", opts);

    let tracks: Vec<(ReplayGain, EbuR128)> = opts
        .files
        .iter()
        .map(|x| track_rg(x, opts.pre_gain).unwrap())
        .collect();

    let album: Option<ReplayGain> = if opts.do_album {
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

    match opts.mode {
        options::Mode::WriteExtended => todo!(),
        options::Mode::Write => opts.files.iter().zip(tracks).for_each(|(path, rg)| {
            tagger::write_tags(
                path,
                rg,
                album,
                false,
                &opts.unit,
                opts.lowercase,
                opts.strip,
                opts.id3v2version,
            )
        }),
        options::Mode::Noop => { /* no-op */ }
        options::Mode::Delete => todo!(),
    }
}
