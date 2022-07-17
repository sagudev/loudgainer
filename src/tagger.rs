use std::path::Path;

use crate::options::Id3v2version;
use crate::replay_gain::ReplayGain;

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
}

pub fn delete_tags<P: AsRef<Path>>(path: P) {}
