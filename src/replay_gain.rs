use std::path::Path;

use ebur128::{EbuR128, Error, Mode};
use log::warn;

use crate::audio::Audi;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplayGain {
    // This field must be
    pub gain: f64,
    // This is optional in rg1
    pub peak: f64,
    // This two are only used by loudgain
    pub loudness_range: f64,
    pub loudness_reference: f64,
    // This field is not written to files
    pub loudness: f64,
}

impl ReplayGain {
    pub fn display(&self, unit: String) {
        println!("Loudness: {:8.2} LUFS", self.loudness);
        println!("Range: {:8.2} {unit}", self.loudness_range);
        println!(
            "Peak: {:8.6} ({:8.6} dBTP)",
            self.peak,
            lufs_to_dbtp(self.peak)
        );
        println!("Gain: {:8.2} {unit}", self.gain)
    }

    /// Detect clip and prevent it if requested
    pub fn clipper(&self, max_true_peak_level: f64, warn: bool, prevent: bool) -> Self {
        let peak_limit = dbtp_to_lufs(max_true_peak_level);
        // new peak after gain
        let new_peak = dbtp_to_lufs(self.gain) * self.peak;

        if new_peak > peak_limit {
            if prevent {
                let new_new_peak = new_peak.min(peak_limit);
                return Self {
                    gain: self.gain - lufs_to_dbtp(new_peak / new_new_peak),
                    ..*self
                };
            } else if warn {
                warn!("The track will clip!");
            }
        }

        *self
    }
}

impl std::fmt::Display for ReplayGain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Loudness: {:8.2} LUFS", self.loudness)?;
        writeln!(f, "Range: {:8.2}", self.loudness_range)?;
        writeln!(
            f,
            "Peak: {:8.6} ({:8.6} dBTP)",
            self.peak,
            lufs_to_dbtp(self.peak)
        )?;
        writeln!(f, "Gain: {:8.2}", self.gain)
    }
}

/// Calculates ReplayGain(2) with -18.00 LUFS
pub fn track_rg<P: AsRef<Path>>(path: P, pregain: f64) -> Result<(ReplayGain, EbuR128), Error> {
    use crate::audio::Audio;

    let audi = Audi::from_path(path);

    // prepare ebur128
    let mut e = EbuR128::new(
        audi.channels,
        audi.sample_rate as u32,
        //Mode::S | Mode::I | Mode::LRA | Mode::TRUE_PEAK | Mode::SAMPLE_PEAK,
        Mode::I | Mode::LRA | Mode::TRUE_PEAK,
    )?;

    match audi.audio {
        Audio::S16(x) => e.add_frames_i16(&x)?,
        Audio::S32(x) => e.add_frames_i32(&x)?,
        Audio::F32(x) => e.add_frames_f32(&x)?,
        Audio::F64(x) => e.add_frames_f64(&x)?,
    }

    let global = e.loudness_global()?;
    let range = e.loudness_range()?;
    let peak = (0..audi.channels)
        .map(|i| e.true_peak(i).unwrap())
        .reduce(f64::max)
        .unwrap();

    Ok((
        ReplayGain {
            gain: lufs_to_rg(global) + pregain,
            peak,
            loudness: global,
            loudness_range: range,
            loudness_reference: lufs_to_rg(-pregain),
        },
        e,
    ))
}

pub fn album_rg(scans: &[(ReplayGain, EbuR128)], pregain: f64) -> Result<ReplayGain, Error> {
    let global = EbuR128::loudness_global_multiple(scans.iter().map(|(_, e)| e))?;
    let range = EbuR128::loudness_range_multiple(scans.iter().map(|(_, e)| e))?;

    let peak = scans
        .iter()
        .map(|(rg, _)| rg.peak)
        .reduce(f64::max)
        .unwrap();

    Ok(ReplayGain {
        gain: lufs_to_rg(global) + pregain,
        peak,
        loudness: global,
        loudness_range: range,
        loudness_reference: lufs_to_rg(-pregain),
    })
}

#[inline]
pub(crate) fn lufs_to_rg(l: f64) -> f64 {
    -18.0 - l
}

#[inline]
/// The equation to convert to dBTP is: 20 * log10(n)
pub(crate) fn lufs_to_dbtp(n: f64) -> f64 {
    20.0 * (n).log10()
}

#[inline]
/// The equation to convert to LUFS is: 10 ** (n / 20.0)
pub(crate) fn dbtp_to_lufs(n: f64) -> f64 {
    10.0_f64.powf(n / 20.0)
}
