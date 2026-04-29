use super::*;

#[test]
fn downmix_stereo_averages_channels() {
    // Interleaved stereo: [L0, R0, L1, R1, L2, R2]
    let input = vec![1.0, 3.0, 0.0, 0.0, -1.0, 1.0];
    let out = downmix_to_mono(&input, 2);
    assert_eq!(out, vec![2.0, 0.0, 0.0]);
}

#[test]
fn downmix_mono_is_identity() {
    let input = vec![0.1, 0.2, 0.3];
    let out = downmix_to_mono(&input, 1);
    assert_eq!(out, input);
}

#[test]
fn resample_same_rate_is_identity() {
    let input = vec![0.0, 0.5, 1.0, 0.5, 0.0];
    let out = resample_linear(&input, 16_000, 16_000);
    assert_eq!(out, input);
}

#[test]
fn resample_downsample_shrinks_length() {
    // 48kHz → 16kHz is a 3x downsample: 300 samples → 100.
    let input: Vec<f32> = (0..300).map(|i| i as f32 / 300.0).collect();
    let out = resample_linear(&input, 48_000, 16_000);
    assert_eq!(out.len(), 100);
}

#[test]
fn resample_preserves_start_value() {
    let input: Vec<f32> = (0..300).map(|i| i as f32 / 300.0).collect();
    let out = resample_linear(&input, 48_000, 16_000);
    assert!((out[0] - 0.0).abs() < 0.01);
}

#[test]
fn wav_encode_produces_valid_header() {
    let samples = vec![0.0_f32; 16_000]; // 1 second of silence
    let wav = encode_wav_f32_mono_16khz(&samples).unwrap();
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert!(wav.len() > 44);
}

#[test]
fn peak_and_rms_silence_is_zero() {
    let silence = vec![0.0_f32; 16_000];
    let (peak, rms) = peak_and_rms(&silence);
    assert_eq!(peak, 0.0);
    assert_eq!(rms, 0.0);
}

#[test]
fn peak_and_rms_one_loud_sample_lifts_peak() {
    // 16k samples of silence + one loud sample → peak should track the
    // loud sample regardless of how short it is.
    let mut buf = vec![0.0_f32; 16_000];
    buf[100] = 0.7;
    let (peak, rms) = peak_and_rms(&buf);
    assert!((peak - 0.7).abs() < 1e-6);
    // RMS is dominated by the silence, so it stays tiny.
    assert!(rms < 0.01);
}

#[test]
fn peak_and_rms_constant_amplitude() {
    let buf = vec![0.5_f32; 1000];
    let (peak, rms) = peak_and_rms(&buf);
    assert!((peak - 0.5).abs() < 1e-6);
    assert!((rms - 0.5).abs() < 1e-6);
}

#[test]
fn peak_and_rms_negative_amplitude_uses_abs() {
    let buf = vec![-0.3_f32; 100];
    let (peak, _) = peak_and_rms(&buf);
    assert!((peak - 0.3).abs() < 1e-6);
}

#[test]
fn peak_and_rms_empty_returns_zero() {
    let (peak, rms) = peak_and_rms(&[]);
    assert_eq!(peak, 0.0);
    assert_eq!(rms, 0.0);
}
