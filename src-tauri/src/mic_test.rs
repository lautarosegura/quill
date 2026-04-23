//! Live mic level probe used by Settings → Micrófono for the VU meter.
//!
//! When the user clicks "Probar micrófono" the frontend calls
//! [`commands::devices::start_mic_test`], which starts a cpal input stream
//! on a dedicated std thread. The stream computes RMS level per callback
//! chunk and a ~30 Hz emitter loop forwards the value to the frontend as a
//! `mic_level` Tauri event (`{ value: f32 }`, 0.0–1.0).
//!
//! ## Why a dedicated thread?
//! `cpal::Stream` is `!Send` on Windows, so it can't live in an async
//! runtime state. The thread below owns the stream; a small mpsc channel
//! lets commands tell it to shut down.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tauri::{AppHandle, Emitter};

const EMIT_EVENT: &str = "mic_level";
const EMIT_INTERVAL: Duration = Duration::from_millis(33); // ~30 Hz

pub struct MicTestController {
    stop_tx: Option<mpsc::Sender<()>>,
}

impl Default for MicTestController {
    fn default() -> Self {
        Self::new()
    }
}

impl MicTestController {
    pub fn new() -> Self {
        Self { stop_tx: None }
    }

    /// Starts the probe (idempotent: replaces any running probe). Accepts a
    /// device *name* matching what `list_mic_devices` returns; `None` uses the
    /// system default input.
    pub fn start(&mut self, app: AppHandle, device_name: Option<String>) {
        self.stop();
        let (tx, rx) = mpsc::channel::<()>();
        self.stop_tx = Some(tx);

        thread::spawn(move || {
            let host = cpal::default_host();
            let device = match device_name.as_deref() {
                Some(name) => host
                    .input_devices()
                    .ok()
                    .and_then(|mut iter| iter.find(|d| d.name().ok().as_deref() == Some(name)))
                    .or_else(|| host.default_input_device()),
                None => host.default_input_device(),
            };
            let Some(device) = device else {
                log::warn!("mic_test: no input device available");
                return;
            };
            let Ok(cfg) = device.default_input_config() else {
                log::warn!("mic_test: no default input config");
                return;
            };

            let level = Arc::new(Mutex::new(0.0_f32));
            let level_cb = Arc::clone(&level);
            let fmt = cfg.sample_format();
            let cfg_stream = cfg.config();
            let err_fn = |e| log::warn!("mic_test stream error: {e}");

            let stream = match fmt {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &cfg_stream,
                    move |data: &[f32], _| {
                        *level_cb.lock().unwrap() = rms_f32(data);
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &cfg_stream,
                    move |data: &[i16], _| {
                        *level_cb.lock().unwrap() = rms_i16(data);
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &cfg_stream,
                    move |data: &[u16], _| {
                        *level_cb.lock().unwrap() = rms_u16(data);
                    },
                    err_fn,
                    None,
                ),
                other => {
                    log::warn!("mic_test: unsupported sample format {:?}", other);
                    return;
                }
            };
            let Ok(stream) = stream else {
                log::warn!("mic_test: build_input_stream failed");
                return;
            };
            if stream.play().is_err() {
                log::warn!("mic_test: stream.play() failed");
                return;
            }

            // Emitter loop: ticks every ~33 ms, exits when `stop()` sends.
            loop {
                match rx.recv_timeout(EMIT_INTERVAL) {
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        let _ = app.emit(EMIT_EVENT, 0.0_f32);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let lvl = *level.lock().unwrap();
                        let _ = app.emit(EMIT_EVENT, lvl);
                    }
                }
            }
            // `stream` is dropped here, which pauses cpal cleanly.
        });
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn rms_f32(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt().clamp(0.0, 1.0)
}

fn rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples
        .iter()
        .map(|&s| {
            let f = s as f32 / i16::MAX as f32;
            f * f
        })
        .sum();
    (sum_sq / samples.len() as f32).sqrt().clamp(0.0, 1.0)
}

fn rms_u16(samples: &[u16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples
        .iter()
        .map(|&s| {
            // Center u16 on 0 before computing power.
            let f = (s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0);
            f * f
        })
        .sum();
    (sum_sq / samples.len() as f32).sqrt().clamp(0.0, 1.0)
}
