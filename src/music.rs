use std::fs::File;
use std::path::Path;

use simplemad::Decoder;

/// Returns Vec of samples, and duration in seconds.
pub fn read_mp3_file(file_name: String) -> (Vec<f32>, f32) {
	let path_string = format!("music/{}", file_name);
	let path = Path::new(&path_string);
	let file = File::open(&path).unwrap();
	let decoder = Decoder::decode(file).unwrap();

	let mut samples = Vec::new();
	let mut duration = 0.0;
	for decoding_result in decoder {
	    match decoding_result {
	        Err(e) => println!("Error: {:?}", e),
	        Ok(frame) => {
	        	duration += frame.duration.as_secs() as f32 + (frame.duration.subsec_nanos() as f32 / 1000000000.0);
	        	for i in 0..frame.samples[0].len() {
		        	let l_sample = frame.samples[0][i].to_f32();
		        	let r_sample = frame.samples[1][i].to_f32();
		        	let avg_sample = (l_sample + r_sample) / 2.0;
	        		samples.push(avg_sample);
	        	}
	        },
	    }
	}

	(samples, duration)
}
