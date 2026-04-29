//! AudioRecorder — cross-platform microphone capture via cpal, producing
//! 16 kHz mono f32 WAV bytes ready for Whisper.

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use hound::{WavSpec, WavWriter};

use crate::error::QuillError;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const PRE_ROLL_MS: u32 = 150;

/// Owns the cpal stream and the shared buffer of captured samples.
pub struct AudioRecorder {
    inner: Arc<Mutex<RecorderState>>,
    _stream: Stream,
    source_sample_rate: u32,
    source_channels: u16,
}

// Safety: cpal::Stream is !Send on Windows because of WASAPI COM constraints,
// but we only ever touch it from the thread that built it. We wrap AudioRecorder
// in Arc downstream and the stream handle is only dropped on program exit.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

struct RecorderState {
    /// Rolling pre-roll buffer (always filling). Holds the last PRE_ROLL_MS
    /// of source-rate interleaved f32 samples.
    preroll: Vec<f32>,
    /// Main capture buffer, populated only while `active == true`.
    capture: Vec<f32>,
    active: bool,
    preroll_max_len: usize,
}

impl AudioRecorder {
    /// Opens the system default input device and starts the rolling buffer.
    pub fn new() -> Result<Self, QuillError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| QuillError::Audio("no default input device".into()))?;
        let supported = device
            .default_input_config()
            .map_err(|e| QuillError::Audio(format!("no default input config: {e}")))?;
        let config: StreamConfig = supported.clone().into();
        let sample_format = supported.sample_format();
        let source_sample_rate = config.sample_rate.0;
        let source_channels = config.channels;

        let preroll_max_len = ((source_sample_rate as u64 * PRE_ROLL_MS as u64) / 1000) as usize
            * source_channels as usize;

        let inner = Arc::new(Mutex::new(RecorderState {
            preroll: Vec::with_capacity(preroll_max_len),
            capture: Vec::new(),
            active: false,
            preroll_max_len,
        }));

        let stream = match sample_format {
            SampleFormat::F32 => build_stream::<f32>(&device, &config, Arc::clone(&inner))?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config, Arc::clone(&inner))?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config, Arc::clone(&inner))?,
            other => {
                return Err(QuillError::Audio(format!(
                    "unsupported sample format: {other:?}"
                )));
            }
        };
        stream
            .play()
            .map_err(|e| QuillError::Audio(format!("stream play failed: {e}")))?;

        Ok(Self {
            inner,
            _stream: stream,
            source_sample_rate,
            source_channels,
        })
    }

    pub fn start_recording(&self) {
        let mut st = self.inner.lock().unwrap();
        st.capture.clear();
        // Clone preroll first to satisfy the borrow checker (can't hold both
        // &st.preroll and &mut st.capture at once).
        let preroll = st.preroll.clone();
        st.capture.extend_from_slice(&preroll);
        st.active = true;
    }

    pub fn stop_recording(&self) -> Result<AudioCapture, QuillError> {
        let mut st = self.inner.lock().unwrap();
        st.active = false;
        let raw = std::mem::take(&mut st.capture);
        drop(st);

        let mono = downmix_to_mono(&raw, self.source_channels);
        let resampled = resample_linear(&mono, self.source_sample_rate, TARGET_SAMPLE_RATE);
        let (peak, rms) = peak_and_rms(&resampled);
        // Subtract the pre-roll (always present) from duration calculation —
        // the user's intentional speech window starts at the start_recording
        // call, not 150ms earlier.
        let preroll_samples = (TARGET_SAMPLE_RATE as u64 * PRE_ROLL_MS as u64) / 1000;
        let speech_samples = resampled.len().saturating_sub(preroll_samples as usize);
        let duration_ms = (speech_samples as u64 * 1000) / TARGET_SAMPLE_RATE as u64;
        let wav_bytes = encode_wav_f32_mono_16khz(&resampled)?;
        Ok(AudioCapture {
            wav_bytes,
            peak,
            rms,
            duration_ms,
        })
    }
}

/// Stats + bytes from a finished capture. The orchestrator inspects `peak`
/// to gate transcription: a recording with no voice activity (e.g. an
/// accidental hotkey tap) goes straight to Idle without burning a Whisper
/// invocation that would hallucinate output from silence.
#[derive(Debug, Clone)]
pub struct AudioCapture {
    pub wav_bytes: Vec<u8>,
    /// Maximum absolute sample amplitude across the capture, in [0, 1].
    /// Robust to short pauses between syllables (one loud sample lifts it),
    /// so it cleanly distinguishes "no voice at all" from "I said one word".
    pub peak: f32,
    /// Root-mean-square amplitude across the whole capture, in [0, 1].
    /// Lower than `peak` for any real speech because of inter-syllable
    /// silence; useful for diagnostics but `peak` is what the silence
    /// gate keys on.
    pub rms: f32,
    /// Speech duration in milliseconds, EXCLUDING the rolling pre-roll
    /// the recorder always prepends. Reflects the user's intentional
    /// hold time.
    pub duration_ms: u64,
}

fn peak_and_rms(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let mut peak: f32 = 0.0;
    let mut sum_sq: f64 = 0.0;
    for &s in samples {
        let a = s.abs();
        if a > peak {
            peak = a;
        }
        sum_sq += (s as f64) * (s as f64);
    }
    let rms = (sum_sq / samples.len() as f64).sqrt() as f32;
    (peak, rms)
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    state: Arc<Mutex<RecorderState>>,
) -> Result<Stream, QuillError>
where
    T: cpal::Sample + cpal::SizedSample + 'static,
    f32: cpal::FromSample<T>,
{
    let err_fn = |e| eprintln!("audio stream error: {e}");
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut st = state.lock().unwrap();
                let floats: Vec<f32> = data
                    .iter()
                    .copied()
                    .map(cpal::Sample::to_sample::<f32>)
                    .collect();

                // Circular pre-roll update.
                let needed = (st.preroll.len() + floats.len()).saturating_sub(st.preroll_max_len);
                if needed > 0 {
                    st.preroll.drain(..needed);
                }
                st.preroll.extend_from_slice(&floats);

                if st.active {
                    st.capture.extend_from_slice(&floats);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| QuillError::Audio(format!("build_input_stream failed: {e}")))?;
    Ok(stream)
}

fn downmix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    let frames = samples.len() / ch;
    let mut out = Vec::with_capacity(frames);
    for i in 0..frames {
        let sum: f32 = (0..ch).map(|c| samples[i * ch + c]).sum();
        out.push(sum / ch as f32);
    }
    out
}

/// Linear resampling — good enough for speech (not ideal for music).
fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 * ratio;
        let idx = src as usize;
        let frac = src - idx as f64;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac as f32);
    }
    out
}

fn encode_wav_f32_mono_16khz(samples: &[f32]) -> Result<Vec<u8>, QuillError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut buf = Cursor::new(Vec::<u8>::new());
    {
        let mut writer = WavWriter::new(&mut buf, spec)
            .map_err(|e| QuillError::Audio(format!("wav writer: {e}")))?;
        for s in samples {
            writer
                .write_sample(*s)
                .map_err(|e| QuillError::Audio(format!("wav write: {e}")))?;
        }
        writer
            .finalize()
            .map_err(|e| QuillError::Audio(format!("wav finalize: {e}")))?;
    }
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests;
