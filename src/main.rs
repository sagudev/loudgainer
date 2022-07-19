use ebur128::EbuR128;
use log::debug;

use crate::replay_gain::{album_rg, track_rg, ReplayGain};

mod audio;
mod options;
mod replay_gain;
mod tagger;

fn main() {
    let opts = options::parse_arguments();
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    debug!("{:#?}", opts);

    match opts.output {
        options::OutputMode::Human => {println!("Scanning all files.")},
        options::OutputMode::Old => println!("File\tMP3 gain\tdB gain\tMax Amplitude\tMax global_gain\tMin global_gain"),
        options::OutputMode::New => println!("File\tLoudness\tRange\tTrue_Peak\tTrue_Peak_dBTP\tReference\tWill_clip\tClip_prevent\tGain\tNew_Peak\tNew_Peak_dBTP"),
    };

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

    for (path, (rg, _)) in opts.files.iter().zip(tracks) {
        // check clipping and maybe prevent it
        let rg = rg.clipper(
            opts.max_true_peak_level,
            opts.warn_clip,
            opts.clip_prevention,
        );

        // do requested stuff on file
        match opts.mode {
            options::Mode::WriteExtended => tagger::write_tags(
                path,
                rg,
                album,
                true,
                &opts.unit,
                opts.lowercase,
                opts.strip,
                opts.id3v2version,
            ),
            options::Mode::Write => tagger::write_tags(
                path,
                rg,
                album,
                false,
                &opts.unit,
                opts.lowercase,
                opts.strip,
                opts.id3v2version,
            ),
            options::Mode::Noop => { /* no-op */ }
            options::Mode::Delete => todo!(),
        }

        match opts.output {
            options::OutputMode::Human => {
                rg.display(&opts.unit)
            },
            options::OutputMode::Old => todo!("File\tMP3 gain\tdB gain\tMax Amplitude\tMax global_gain\tMin global_gain"),
            options::OutputMode::New => todo!("File\tLoudness\tRange\tTrue_Peak\tTrue_Peak_dBTP\tReference\tWill_clip\tClip_prevent\tGain\tNew_Peak\tNew_Peak_dBTP"),
        };
    }
}
