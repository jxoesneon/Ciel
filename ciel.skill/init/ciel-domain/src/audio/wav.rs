//! WAV I/O shared by the audio ports: 16/24-bit PCM and 32-bit IEEE float
//! writers plus a `wave`-module-equivalent reader producing normalized f32.

use std::fs;
use std::path::Path;

/// `procedural_audio_generator.py::write_wav_32bit_float`
pub fn write_wav_32bit_float(filename: &str, samples: &[(f64, f64)], sample_rate: u32) {
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 32;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = samples.len() as u32 * block_align as u32;

    if let Some(parent) = Path::new(&crate::common::py::abspath(filename)).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = Vec::with_capacity(44 + data_size as usize);
    f.extend_from_slice(b"RIFF");
    f.extend_from_slice(&(36 + data_size).to_le_bytes());
    f.extend_from_slice(b"WAVE");
    f.extend_from_slice(b"fmt ");
    f.extend_from_slice(&16u32.to_le_bytes());
    f.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
    f.extend_from_slice(&num_channels.to_le_bytes());
    f.extend_from_slice(&sample_rate.to_le_bytes());
    f.extend_from_slice(&byte_rate.to_le_bytes());
    f.extend_from_slice(&block_align.to_le_bytes());
    f.extend_from_slice(&bits_per_sample.to_le_bytes());
    f.extend_from_slice(b"data");
    f.extend_from_slice(&data_size.to_le_bytes());
    for &(l, r) in samples {
        let cl_l = (l as f32).clamp(-1.0, 1.0);
        let cl_r = (r as f32).clamp(-1.0, 1.0);
        f.extend_from_slice(&cl_l.to_le_bytes());
        f.extend_from_slice(&cl_r.to_le_bytes());
    }
    let _ = fs::write(filename, f);
}

/// `procedural_audio_generator.py::write_wav_24bit_pcm`
pub fn write_wav_24bit_pcm(filename: &str, samples: &[(f64, f64)], sample_rate: u32) {
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 24;
    let bytes_per_sample = 3u16;
    let block_align = num_channels * bytes_per_sample;
    let data_size = samples.len() as u32 * block_align as u32;

    if let Some(parent) = Path::new(&crate::common::py::abspath(filename)).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = Vec::with_capacity(44 + data_size as usize);
    f.extend_from_slice(b"RIFF");
    f.extend_from_slice(&(36 + data_size).to_le_bytes());
    f.extend_from_slice(b"WAVE");
    f.extend_from_slice(b"fmt ");
    f.extend_from_slice(&16u32.to_le_bytes());
    f.extend_from_slice(&1u16.to_le_bytes()); // PCM
    f.extend_from_slice(&num_channels.to_le_bytes());
    f.extend_from_slice(&sample_rate.to_le_bytes());
    f.extend_from_slice(&(sample_rate * block_align as u32).to_le_bytes());
    f.extend_from_slice(&block_align.to_le_bytes());
    f.extend_from_slice(&bits_per_sample.to_le_bytes());
    f.extend_from_slice(b"data");
    f.extend_from_slice(&data_size.to_le_bytes());
    let max_int24: i64 = 8388607;
    for &(l, r) in samples {
        for s in [l, r] {
            let v = (s.clamp(-1.0, 1.0) * max_int24 as f64) as i64;
            let b = (v as i32).to_le_bytes();
            f.extend_from_slice(&b[..3]);
        }
    }
    let _ = fs::write(filename, f);
}

/// `aaa_audio_generator.py::write_wav` — mono with soft clip, 16/24-bit PCM.
pub fn write_wav_mono(filename: &str, samples: &[f64], sample_rate: u32, bit_depth: u16) {
    let channels = 1u16;
    let sampwidth = bit_depth / 8;
    let block_align = channels * sampwidth;
    let data_size = samples.len() as u32 * block_align as u32;

    let mut out = Vec::with_capacity(44 + data_size as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_size).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * block_align as u32).to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bit_depth.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        let sat = if s.abs() > 0.001 {
            (s + 0.25 * s * s) / (1.0 + 0.4 * s.abs())
        } else {
            s
        };
        if bit_depth == 16 {
            let v = (sat * 32760.0).clamp(-32767.0, 32767.0) as i64;
            out.extend_from_slice(&(v as i16).to_le_bytes());
        } else if bit_depth == 24 {
            let v = (sat * 8388600.0).clamp(-8388607.0, 8388607.0) as i64;
            let b = (v as i32).to_le_bytes();
            out.extend_from_slice(&b[..3]);
        }
    }
    println!(
        "[OK] Baked {} ({:.2}s, {}-bit)",
        filename,
        samples.len() as f64 / sample_rate as f64,
        bit_depth
    );
    let _ = fs::write(filename, out);
}

/// Python `wave` module reader → (mono-mixed normalized samples, fs, channels).
/// Mirrors `audio_manifest_extractor.load_wav_pcm` semantics.
pub fn load_wav_pcm(file_path: &str) -> Result<(Vec<f64>, u32, usize), String> {
    if !Path::new(file_path).exists() {
        return Err(format!("File not found: {}", file_path));
    }
    let data = fs::read(file_path).map_err(|e| e.to_string())?;
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(format!("Not a RIFF/WAVE file: {}", file_path));
    }

    let mut pos = 12usize;
    let mut fmt_tag = 1u16;
    let mut n_channels = 1usize;
    let mut fs_rate = 44100u32;
    let mut sampwidth = 2usize;
    let mut data_off = 0usize;
    let mut data_len = 0usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        let body = pos + 8;
        if id == b"fmt " && body + 16 <= data.len() {
            fmt_tag = u16::from_le_bytes([data[body], data[body + 1]]);
            n_channels = u16::from_le_bytes([data[body + 2], data[body + 3]]) as usize;
            fs_rate = u32::from_le_bytes([
                data[body + 4],
                data[body + 5],
                data[body + 6],
                data[body + 7],
            ]);
            let bits = u16::from_le_bytes([data[body + 14], data[body + 15]]) as usize;
            sampwidth = bits / 8;
        } else if id == b"data" {
            data_off = body;
            data_len = size.min(data.len().saturating_sub(body));
        }
        pos = body + size + (size % 2); // RIFF pad byte
    }
    if data_len == 0 {
        return Err("WAV file contains no data chunk.".to_string());
    }
    let raw = &data[data_off..data_off + data_len];

    let mut samples: Vec<f64> = Vec::new();
    if sampwidth == 2 && fmt_tag == 1 {
        for c in raw.chunks_exact(2) {
            samples.push(i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0);
        }
    } else if sampwidth == 4 && fmt_tag == 3 {
        for c in raw.chunks_exact(4) {
            samples.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64);
        }
    } else if sampwidth == 4 {
        for c in raw.chunks_exact(4) {
            samples.push(i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64 / 2147483648.0);
        }
    } else if sampwidth == 3 {
        for c in raw.chunks_exact(3) {
            let v = i32::from_le_bytes([c[0], c[1], c[2], if c[2] & 0x80 != 0 { 0xFF } else { 0 }]);
            samples.push(v as f64 / 8388608.0);
        }
    } else if sampwidth == 1 {
        for &b in raw {
            samples.push((b as f64 - 128.0) / 128.0);
        }
    } else {
        for c in raw.chunks_exact(2) {
            samples.push(i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0);
        }
    }

    // Convert to mono if multi-channel
    if n_channels > 1 && samples.len() >= n_channels {
        let mut mono = Vec::with_capacity(samples.len() / n_channels);
        let mut i = 0usize;
        while i + n_channels <= samples.len() {
            mono.push(samples[i..i + n_channels].iter().sum::<f64>() / n_channels as f64);
            i += n_channels;
        }
        return Ok((mono, fs_rate, n_channels));
    }
    Ok((samples, fs_rate, n_channels))
}

/// `scipy.io.wavfile.read` equivalent used by the audiogram: returns
/// normalized interleaved channels + whether the source is true stereo.
pub fn load_wav_stereo(file_path: &str) -> Result<(Vec<f64>, Vec<f64>, u32, bool), String> {
    if !Path::new(file_path).exists() {
        return Err(format!("Audio file not found: {}", file_path));
    }
    let data = fs::read(file_path).map_err(|e| e.to_string())?;
    if data.len() < 44 || &data[0..4] != b"RIFF" {
        return Err(format!("Not a RIFF/WAVE file: {}", file_path));
    }
    let mut pos = 12usize;
    let mut fmt_tag = 1u16;
    let mut n_channels = 1usize;
    let mut fs_rate = 44100u32;
    let mut sampwidth = 2usize;
    let mut data_off = 0usize;
    let mut data_len = 0usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        let body = pos + 8;
        if id == b"fmt " && body + 16 <= data.len() {
            fmt_tag = u16::from_le_bytes([data[body], data[body + 1]]);
            n_channels = u16::from_le_bytes([data[body + 2], data[body + 3]]) as usize;
            fs_rate = u32::from_le_bytes([
                data[body + 4],
                data[body + 5],
                data[body + 6],
                data[body + 7],
            ]);
            let bits = u16::from_le_bytes([data[body + 14], data[body + 15]]) as usize;
            sampwidth = bits / 8;
        } else if id == b"data" {
            data_off = body;
            data_len = size.min(data.len().saturating_sub(body));
        }
        pos = body + size + (size % 2);
    }
    let raw = &data[data_off..data_off + data_len];

    let frame_w = (n_channels * sampwidth).max(1);
    let n_frames = raw.len() / frame_w;
    let mut left = vec![0.0f64; n_frames];
    let mut right = vec![0.0f64; n_frames];

    let decode_one = |c: &[u8]| -> f64 {
        if sampwidth == 2 {
            i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0
        } else if sampwidth == 4 && fmt_tag == 3 {
            f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64
        } else if sampwidth == 4 {
            i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64 / 2147483648.0
        } else if sampwidth == 3 {
            let v = i32::from_le_bytes([c[0], c[1], c[2], if c[2] & 0x80 != 0 { 0xFF } else { 0 }]);
            v as f64 / 8388608.0
        } else {
            (c[0] as f64 - 128.0) / 128.0
        }
    };

    for f in 0..n_frames {
        let base = f * frame_w;
        left[f] = decode_one(&raw[base..base + sampwidth]);
        if n_channels >= 2 {
            right[f] = decode_one(&raw[base + sampwidth..base + 2 * sampwidth]);
        } else {
            right[f] = left[f];
        }
    }
    Ok((left, right, fs_rate, n_channels >= 2))
}
