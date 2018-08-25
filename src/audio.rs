use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

// == FFT Notes ==

// Frequencies
// Reference: https://stackoverflow.com/questions/4364823/
// Each bin of FFT output represents frequency [i * sample_rate / sample_count].
// N / 2 will represent the Nyquist frequency.

// Why is it a complex number?
// Reference: https://www.gaussianwaves.com/2015/11/interpreting-fft
// The combination of real and complex represents the amplitude and phase of that frequency bucket.

pub fn compute_fft(source: Vec<f32>) -> Vec<Complex<f32>> {
    let mut input: Vec<Complex<f32>> = vec![Complex::zero(); source.len()];
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); source.len()];

    for i in 0..source.len() {
        input[i] = Complex::new(source[i] as f32, 0.0);
    }

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(source.len());
    fft.process(&mut input, &mut output);

    output
}

pub fn frequency_bins(sample_rate: u32, sample_count: u32) -> Vec<f32> {
    let bin_count = (sample_count / 2) as usize;
    let mut output: Vec<f32> = vec![0.0; bin_count];

    for i in 0..bin_count {
        output[i] = (i as f32) * (sample_rate as f32) / (sample_count as f32);
    }

    output
}

pub fn to_amplitude(input: Vec<Complex<f32>>, amplitude_scalar: f32) -> Vec<f32> {
    let mut output: Vec<f32> = vec![0.0; input.len()];

    for i in 0..input.len() {
        let re = input[i].re;
        let im = input[i].im;
        output[i] = (re * re + im * im).sqrt() / input.len() as f32;
        output[i] *= amplitude_scalar;
    }

    output
}


#[derive(Clone)]
pub struct AudioFrame {
    pub bpm: f32,
    pub low_power: f32,
    pub mid_power: f32,
    pub high_power: f32,
    pub hundred_hz_buckets: [f32; 200],
}
