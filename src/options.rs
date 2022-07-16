use std::{path::PathBuf, process::exit, string::ParseError};

use gumdrop::Options;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// loudgainer is a loudness normalizer that scans music files and calculates loudness-normalized gain and loudness peak values according to the EBU R128 standard, and can optionally write ReplayGain-compatible metadata.
///
/// loudgainer implements a subset of mp3gain's command-line options, which means that it can be used as a drop-in replacement in some situations.
///
/// loudgainer will not modify the actual audio data, but instead just write ReplayGain tags if so requested. It is up to the player to interpret these. (In some players, you need to enable this feature.)
///
/// loudgainer currently supports writing tags to the following file types:
/// FLAC (.flac), Ogg (.ogg, .oga, .spx, .opus), MP2 (.mp2), MP3 (.mp3), MP4 (.mp4, .m4a), ASF/WMA (.asf, .wma), WavPack (.wv), APE (.ape).
///
/// Experimental, use with care: WAV (.wav), AIFF (.aiff, .aif, .snd).
#[derive(Debug, Options)]
struct MyOptions {
    // Contains fi
    #[options(free)]
    files: Vec<String>,

    #[options(help = "Show this help")]
    help: bool,

    #[options(help = "Show version numbers")]
    version: bool,

    #[options(
        short = "r",
        help = "Calculate track gain only (default)",
        default_expr = "true"
    )]
    track: bool,

    #[options(help = "Calculate album gain (and track gain)")]
    album: bool,

    #[options(help = "Ignore clipping warnings")]
    clip: bool,

    #[options(
        short = "k",
        help = "Lower track/album gain to avoid clipping (<= -1 dBTP)"
    )]
    noclip: bool,

    #[options(
        short = "K",
        help = "Avoid clipping; max. true peak level = n dBTP",
        meta = "n"
    )]
    maxtpl: Option<i32>,

    #[options(
        short = "d",
        help = "Apply n dB/LU pre-gain value (-5 for -23 LUFS target)",
        meta = "n"
    )]
    pregain: Option<i32>,

    #[options(
        short = "s",
        help = "
        TAGMODES:
            d: Delete ReplayGain tags from files.
            i: Write ReplayGain 2.0 tags to files. ID3v2 for MP2, MP3, WAV and AIFF; Vorbis Comments for FLAC, Ogg, Speex and Opus; iTunes-type metadata for MP4/M4A; WMA tags for ASF/WMA; APEv2 tags for WavPack and APE.
            e: like '-s i', plus extra tags (reference, ranges).
            l: like '-s e', but LU units instead of dB.
            s: Don't write ReplayGain tags (default).
        "
    )]
    tagmode: Tagmode,

    #[options(
        short = "L",
        help = "Force lowercase 'REPLAYGAIN_*' tags (MP2/MP3/MP4/ASF/WMA/WAV/AIFF only). This is non-standard, but sometimes needed"
    )]
    lowercase: bool,

    #[options(
        short = "S",
        help = "Strip tag types other than ID3v2 from MP2/MP3 files (i.e. ID3v1, APEv2). Strip tag types other than APEv2 from WavPack/APE files (i.e. ID3v1)"
    )]
    striptags: bool,

    #[options(
        short = "I",
        help = "Write ID3v2.N tags to MP2/MP3/WAV/AIFF files (only 3 and 4 are supported)",
        meta = "N"
    )]
    id3v2version: Id3v2version,

    #[options(help = "Database-friendly tab-delimited list output (mp3gain-compatible)")]
    output: bool,

    #[options(
        short = "O",
        long = "output-new",
        help = "Database-friendly new format tab-delimited list output. Ideal for analysis of files if redirected to a CSV file"
    )]
    output_new: bool,

    #[options(help = "Database-friendly tab-delimited list output (mp3gain-compatible)")]
    quiet: bool,
}

#[derive(Debug, Default, PartialEq)]
enum Tagmode {
    /// Delete ReplayGain tags from files.
    D,
    /// Write ReplayGain 2.0 tags to files. ID3v2 for MP2, MP3, WAV and AIFF; Vorbis Comments for FLAC, Ogg, Speex and Opus; iTunes-type metadata for MP4/M4A; WMA tags for ASF/WMA; APEv2 tags for WavPack and APE.
    I,
    /// like '-s i', plus extra tags (reference, ranges).
    E,
    /// like '-s e', but LU units instead of dB.
    L,
    #[default]
    /// Don't write ReplayGain tags.
    S,
}

impl std::str::FromStr for Tagmode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase().chars().next().unwrap() {
            'd' => Ok(Self::D),
            'i' => Ok(Self::I),
            'e' => Ok(Self::E),
            'l' => Ok(Self::L),
            's' => Ok(Self::S),
            _ => panic!("Invalid tag mode!"),
        }
    }
}

#[derive(Debug, Default)]
enum Id3v2version {
    V3,
    #[default]
    V4,
}

impl std::str::FromStr for Id3v2version {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase().chars().next().unwrap() {
            '3' => Ok(Self::V3),
            '4' => Ok(Self::V4),
            _ => panic!("Invalid ID3v2 version; only 3 and 4 are supported."),
        }
    }
}

#[derive(Debug)]
pub enum OutputMode {
    /// output something human-readable
    Human,
    /// output old-style mp3gain-compatible list
    Old,
    /// output new style list: File;Loudness;Range;Gain;Reference;Peak;Peak dBTP;Clipping;Clip-prevent
    New,
}

#[derive(Debug, Default)]
pub enum Mode {
    /// like Write mode, with extra tags (reference, ranges).
    WriteExtended,
    /// Write ReplayGain 2.0 tags to files. ID3v2 for MP2, MP3, WAV and AIFF; Vorbis Comments for FLAC, Ogg, Speex and Opus; iTunes-type metadata for MP4/M4A; WMA tags for ASF/WMA; APEv2 tags for WavPack and APE.
    Write,
    #[default]
    /// Don't write ReplayGain tags.
    Noop,
    /// Delete ReplayGain tags from files.
    Delete,
}

#[derive(Debug)]
pub struct Opts {
    /// files to be processed
    files: Vec<PathBuf>,
    /// output mode
    output: OutputMode,
    /// units: dB or LU
    units: String,
    /// Working Mode (cmd)
    mode: Mode,
    /// pre-gain
    pre_gain: f64,
    /// dBTP; default for -k, as per EBU Tech 3343
    max_true_peak_level: f64,
}

pub fn parse_arguments() -> Opts {
    let opts = MyOptions::parse_args_default_or_exit();
    // process version
    if opts.version {
        println!("loudgainer v{VERSION}");
        println!("ebur128 v0.1.6 based on libebur128 1.2.6");
        exit(0)
    };

    //let mut pre_gain: 0.0,
    //max_true_peak_level: -1.0,

    Opts {
        pre_gain: 0.0,
        max_true_peak_level: -1.0,
        files: opts.files.into_iter().map(|x| x.into()).collect(),
        output: if opts.output {
            OutputMode::Old
        } else if opts.output_new {
            OutputMode::New
        } else {
            OutputMode::Human
        },
        units: if opts.tagmode == Tagmode::L {
            String::from("LU")
        } else {
            String::from("dB")
        },
        mode: match opts.tagmode {
            Tagmode::D => Mode::Delete,
            Tagmode::I => Mode::WriteExtended,
            Tagmode::E => Mode::WriteExtended,
            Tagmode::L => Mode::Write,
            Tagmode::S => Mode::Noop,
        },
    }
}
