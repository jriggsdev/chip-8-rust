use std::io::Cursor;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use anyhow::Result;

/// Controls playing a tone
pub struct AudioPlayer {
    tone: Sound,
    is_playing_tone: bool,
}

impl AudioPlayer {
    /// Builds a new Audio player that will play a tone with the given frequency, duration, and
    /// amplitude
    pub async fn build(frequency: f32, duration: f32, amplitude: f32) -> Self {
        let tone = generate_tone(frequency, duration, amplitude).await
            .expect("Failed to generate tone");

        Self {
            tone,
            is_playing_tone: false,
        }
    }

    /// Plays the tone
    pub fn play_tone(&mut self) {
        if self.is_playing_tone {
            return
        }
        
        self.is_playing_tone = true;
        play_sound(&self.tone, PlaySoundParams { looped: false, volume: 0.2 });
    }

    /// Stops playing the tone
    pub fn stop_tone(&mut self) {
        if !self.is_playing_tone {
            return
        }
        
        self.is_playing_tone = false;
        stop_sound(&self.tone);
    }
}

/// Generates a tone based on the given frequency, duration, and amplitude
async fn generate_tone(frequency: f32, duration: f32, amplitude: f32) -> Result<Sound> {
    let tone_bytes = generate_tone_bytes(frequency, duration, amplitude)?;
    let sound = load_sound_from_bytes(tone_bytes.as_slice()).await?;

    Ok(sound)
}

/// Generates raw bytes for a tone with the provided frequency, duration, and amplitude
// TODO its probably better to just load the tone from a file but I thought this was a neat way to do it
// also don't ask me how this works I vibe coded it
fn generate_tone_bytes(frequency: f32, duration: f32, amplitude: f32) -> Result<Vec<u8>> {
    let spec = hound::WavSpec   {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Generate a sine wave
    let num_samples = (spec.sample_rate as f32 * duration) as usize;
    let mut samples = Vec::new();
    for i in 0..num_samples {
        let t = i as f32 / spec.sample_rate as f32; // Time (in seconds)
        let sample = (amplitude * (t * frequency * 2.0 * std::f32::consts::PI).sin())
            * i16::MAX as f32; // Scale sample to 16-bit range
        samples.push(sample as i16);
    }

    let mut wav_buffer = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut wav_buffer, spec)?;

        for sample in samples {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;
    }

    Ok(wav_buffer.into_inner())
}