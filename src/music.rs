use std::fs::File;
use std::path::Path;

use lewton::inside_ogg::OggStreamReader;

/// Returns Vec of samples, and duration in seconds.
pub fn read_ogg_file(file_name: String) -> (Vec<f32>, f32) {
    let path = Path::new(&file_name);
    let file = File::open(&path).unwrap();

    let mut stream_reader = OggStreamReader::new(file).unwrap();

    let mut samples = Vec::new();
    let mut duration = 0.0;

    let sample_rate = stream_reader.ident_hdr.audio_sample_rate as i32;
    let sample_channels = stream_reader.ident_hdr.audio_channels as f32 *
        stream_reader.ident_hdr.audio_sample_rate as f32;

    while let Some(packet_samples) = stream_reader.read_dec_packet_itl().unwrap() {
        for i in 0..packet_samples.len() {
            let sample_value = packet_samples[i] as f32 / i16::max_value() as f32;
            let clamped_value = f32::max(-1.0, f32::min(1.0, sample_value));
            samples.push(clamped_value);
        }
        duration += packet_samples.len() as f32 / sample_channels;
    }

    (samples, duration)
}
