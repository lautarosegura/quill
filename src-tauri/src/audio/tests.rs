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
