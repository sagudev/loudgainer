use std::{
    fs::File,
    io::{Read, Seek},
};

use log::warn;
use symphonia::core::{
    audio::{AudioBufferRef, SampleBuffer, SignalSpec},
    codecs::DecoderOptions,
    conv::IntoSample,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Duration,
};

pub enum Audio {
    //isnt realy supported in rg
    //S8(Vec<i8>),
    S16(Vec<i16>),
    S32(Vec<i32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

impl Audio {
    pub fn as_audio_ref(&'_ self) -> AudioRef<'_> {
        match self {
            Audio::S16(x) => AudioRef::S16(x),
            Audio::S32(x) => AudioRef::S32(x),
            Audio::F32(x) => AudioRef::F32(x),
            Audio::F64(x) => AudioRef::F64(x),
        }
    }

    pub(crate) fn extend_from_slice(&mut self, samples: AudioRef) {
        match self {
            Audio::S16(x) => x.extend_from_slice(samples.get_i16().unwrap()),
            Audio::S32(x) => x.extend_from_slice(samples.get_i32().unwrap()),
            Audio::F32(x) => x.extend_from_slice(samples.get_f32().unwrap()),
            Audio::F64(x) => x.extend_from_slice(samples.get_f64().unwrap()),
        }
    }

    pub fn to<T>(&self) -> Vec<T>
    where
        T: symphonia::core::conv::FromSample<i16>,
        T: symphonia::core::conv::FromSample<i32>,
        T: symphonia::core::conv::FromSample<f32>,
        T: symphonia::core::conv::FromSample<f64>,
    {
        match self {
            Audio::S16(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Audio::S32(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Audio::F32(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Audio::F64(v) => v.iter().map(|x| (*x).into_sample()).collect(),
        }
    }
}

#[derive(Clone)]
/// Similar as [symphonia::core::audio::AudioBufferRef] with reduced type-set
pub enum AudioRef<'a> {
    //S8(&'a [i8]),
    S16(&'a [i16]),
    S32(&'a [i32]),
    F32(&'a [f32]),
    F64(&'a [f64]),
}

impl<'a> AudioRef<'a> {
    pub fn from_i16(data: &'a [i16]) -> Self {
        Self::S16(data)
    }

    pub fn from_i32(data: &'a [i32]) -> Self {
        Self::S32(data)
    }

    pub fn from_f32(data: &'a [f32]) -> Self {
        Self::F32(data)
    }

    pub fn from_f64(data: &'a [f64]) -> Self {
        Self::F64(data)
    }

    fn get_i16(&self) -> Option<&[i16]> {
        match self {
            AudioRef::S16(x) => Some(x),
            _ => None,
        }
    }

    fn get_i32(&self) -> Option<&[i32]> {
        match self {
            AudioRef::S32(x) => Some(x),
            _ => None,
        }
    }

    fn get_f32(&self) -> Option<&[f32]> {
        match self {
            AudioRef::F32(x) => Some(x),
            _ => None,
        }
    }

    fn get_f64(&self) -> Option<&[f64]> {
        match self {
            AudioRef::F64(x) => Some(x),
            _ => None,
        }
    }

    pub fn to<T>(&self) -> Vec<T>
    where
        T: symphonia::core::conv::FromSample<i16>,
        T: symphonia::core::conv::FromSample<i32>,
        T: symphonia::core::conv::FromSample<f32>,
        T: symphonia::core::conv::FromSample<f64>,
    {
        match *self {
            Self::S16(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Self::S32(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Self::F32(v) => v.iter().map(|x| (*x).into_sample()).collect(),
            Self::F64(v) => v.iter().map(|x| (*x).into_sample()).collect(),
        }
    }
}

impl<'a> AudioRef<'a> {
    pub fn to_owned(&'a self) -> Audio {
        match *self {
            AudioRef::S16(x) => Audio::S16(x.to_owned()),
            AudioRef::S32(x) => Audio::S32(x.to_owned()),
            AudioRef::F32(x) => Audio::F32(x.to_owned()),
            AudioRef::F64(x) => Audio::F64(x.to_owned()),
        }
    }
}

/// AUDIO with some audioinfo
pub struct Audi {
    /// raw audio data
    pub audio: Audio,
    /// Number of channels
    pub channels: u32,
    /// sampling rate in hz
    pub sample_rate: u32,
    /// bit 16 or 24 bit
    pub bits: u8,
}

impl Audi {
    pub fn from_file(mut file: File) -> Self {
        let mut buf = [0; 10];
        file.read_exact(&mut buf).unwrap();
        file.rewind().unwrap();
        if let Some(kind) = infer::get(&buf) {
            if kind.extension() == "flac" {
                return Self::from_flac_file(file);
            } else if kind.matcher_type() == infer::MatcherType::Audio {
                return Self::from_generic_file(file);
            }
        }

        warn!("Fallback to generic Audio reader");
        Self::from_generic_file(file)
    }

    fn from_flac_file(file: File) -> Self {
        let mut r = claxon::FlacReader::new(file).unwrap();
        let streaminfo = r.streaminfo();
        let bits = streaminfo.bits_per_sample as u8;
        let audio = match bits {
            0..=16 => Audio::S16(r.samples().map(|f| f.unwrap() as i16).collect()),
            17..=32 => Audio::S32(r.samples().map(|f| f.unwrap() << (32 - bits)).collect()),
            _ => panic!(""),
        };
        Audi {
            audio,
            channels: streaminfo.channels,
            sample_rate: streaminfo.sample_rate,
            bits,
        }
    }

    fn from_generic_file(file: File) -> Self {
        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a probe hint using the file's extension. [Optional]
        let hint = Hint::new();

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        // Probe the media source.
        let mut probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .unwrap();
        let track = probed.format.default_track().unwrap();
        let track_id = track.id;
        let decode_opts = DecoderOptions { verify: true };

        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();
        let mut sample_buf = None;
        let mut audio: Option<Audio> = None;

        while let Ok(packet) = probed.format.next_packet() {
            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            let audio_buf = decoder.decode(&packet).unwrap();
            // If this is the *first* decoded packet, create a sample buffer matching the
            // decoded audio buffer format.
            if sample_buf.is_none() {
                // Get the audio buffer specification.
                let spec = *audio_buf.spec();

                // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                let duration = audio_buf.capacity() as u64;

                // Create the f32 sample buffer.
                sample_buf = Some(AudioSampleBuffer::new(&audio_buf, duration, spec));
            }

            // Copy the decoded audio buffer into the sample buffer in an interleaved format.
            if let Some(buf) = &mut sample_buf {
                buf.copy_interleaved_ref(audio_buf);

                // The samples may now be access via the `samples()` function.
                if let Some(s) = &mut audio {
                    s.extend_from_slice(buf.samples());
                } else {
                    audio = Some(buf.samples().to_owned())
                }
            }
        }

        let streaminfo = &probed.format.default_track().unwrap().codec_params;

        Audi {
            audio: audio.unwrap(),
            channels: streaminfo.channels.unwrap().count() as u32,
            sample_rate: streaminfo.sample_rate.unwrap(),
            bits: streaminfo.bits_per_sample.unwrap_or(0) as u8,
        }
    }
}

enum AudioSampleBuffer {
    S16(SampleBuffer<i16>),
    S32(SampleBuffer<i32>),
    F32(SampleBuffer<f32>),
    F64(SampleBuffer<f64>),
}

impl AudioSampleBuffer {
    /// create new based on audiobufferref type
    fn new(audio_buf: &AudioBufferRef, duration: Duration, spec: SignalSpec) -> Self {
        match audio_buf {
            AudioBufferRef::U8(_) => Self::S16(SampleBuffer::new(duration, spec)),
            AudioBufferRef::U16(_) => Self::S16(SampleBuffer::new(duration, spec)),
            AudioBufferRef::U24(_) => Self::S32(SampleBuffer::new(duration, spec)),
            AudioBufferRef::U32(_) => Self::S32(SampleBuffer::new(duration, spec)),
            AudioBufferRef::S8(_) => Self::S16(SampleBuffer::new(duration, spec)),
            AudioBufferRef::S16(_) => Self::S16(SampleBuffer::new(duration, spec)),
            AudioBufferRef::S24(_) => Self::S32(SampleBuffer::new(duration, spec)),
            AudioBufferRef::S32(_) => Self::S32(SampleBuffer::new(duration, spec)),
            AudioBufferRef::F32(_) => Self::F32(SampleBuffer::new(duration, spec)),
            AudioBufferRef::F64(_) => Self::F64(SampleBuffer::new(duration, spec)),
        }
    }

    /// Copies all audio data from the source `AudioBufferRef` in interleaved channel order into the
    /// `SampleBuffer`. The two buffers must be equivalent.
    /*
        where
        S: ConvertibleSample,
    */
    pub fn copy_interleaved_ref(&mut self, src: AudioBufferRef) {
        match self {
            Self::S16(s) => s.copy_interleaved_ref(src),
            Self::S32(s) => s.copy_interleaved_ref(src),
            Self::F32(s) => s.copy_interleaved_ref(src),
            Self::F64(s) => s.copy_interleaved_ref(src),
        }
    }

    /// Gets an immutable slice of all written samples.
    pub fn samples(&self) -> AudioRef {
        match self {
            Self::S16(s) => AudioRef::from_i16(s.samples()),
            Self::S32(s) => AudioRef::from_i32(s.samples()),
            Self::F32(s) => AudioRef::from_f32(s.samples()),
            Self::F64(s) => AudioRef::from_f64(s.samples()),
        }
    }
}
