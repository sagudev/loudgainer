use std::path::Path;

use lofty::{Probe, TagType, TaggedFile};
use log::warn;

use crate::options::Id3v2version;
use crate::replay_gain::ReplayGain;

const TAGS: [&str; 9] = [
    "REPLAYGAIN_TRACK_GAIN",
    "REPLAYGAIN_TRACK_PEAK",
    "REPLAYGAIN_TRACK_RANGE",
    "REPLAYGAIN_ALBUM_GAIN",
    "REPLAYGAIN_ALBUM_PEAK",
    "REPLAYGAIN_ALBUM_RANGE",
    "REPLAYGAIN_REFERENCE_LOUDNESS",
    "R128_TRACK_GAIN",
    "R128_ALBUM_GAIN",
];

// this is where we store the RG tags in MP4/M4A files
const RG_ATOM: &str = "----:com.apple.iTunes:";

pub fn write_tags<P: AsRef<Path>>(
    path: P,
    track_rg: ReplayGain,
    album_rg: Option<ReplayGain>,
    extended: bool,
    unit: &str,
    lowercase: bool,
    strip: bool,
    id3v2version: Id3v2version,
) {
    let mut tagger = get_tagger(path);
    if strip {
        tagger.delete_tags();
    }
}

pub fn delete_tags<P: AsRef<Path>>(path: P) {
    let mut tagger = get_tagger(path);
    tagger.delete_tags();
}

fn get_tagger<P: AsRef<Path>>(path: P) -> Tagger {
    match path
        .as_ref()
        .extension()
        .unwrap()
        .to_ascii_lowercase()
        .to_string_lossy()
        .as_ref()
    {
        "flac" => Tagger::Flacer(metaflac::Tag::read_from_path(path).unwrap()),
        _ => {
            warn!("Using generic tegger");
            let mut probe = Probe::open(path).unwrap();
            if probe.file_type().is_none() {
                probe = probe.guess_file_type().expect("Error: Bad file provided!");
            }
            let tagged_file = probe.read(true).unwrap();
            Tagger::Generic(tagged_file)
        }
    }
}

enum Tagger {
    Flacer(metaflac::Tag),
    Generic(TaggedFile),
}

impl Tagger {
    fn delete_tags(&mut self) {
        match self {
            Tagger::Flacer(t) => {
                for tag in TAGS {
                    t.remove_vorbis(tag);
                }
            }
            Tagger::Generic(t) => {
                let vtt: Vec<TagType> = t.tags().iter().map(|x| x.tag_type()).collect();
                for tt in vtt {
                    let ttt = t.tag_mut(&tt).unwrap();
                    match tt {
                        TagType::APE => todo!(),
                        TagType::ID3v1 => todo!(),
                        TagType::ID3v2 => todo!(),
                        TagType::MP4ilst => {
                            for tag in TAGS {
                                ttt.remove_key(&lofty::ItemKey::Unknown(
                                    RG_ATOM.to_ascii_uppercase() + tag,
                                ));
                            }
                        }
                        TagType::VorbisComments | TagType::RIFFInfo | TagType::AIFFText => {
                            for tag in TAGS {
                                ttt.remove_key(&lofty::ItemKey::Unknown(tag.to_owned()));
                                ttt.remove_key(&lofty::ItemKey::Unknown(tag.to_ascii_lowercase()));
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
        }
    }

    fn set_album_tags(&mut self, unit: &str) {}

    fn set_track_tags(&mut self, extended: bool, unit: &str) {}
}
