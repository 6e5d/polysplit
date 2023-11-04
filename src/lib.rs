pub mod poly_host;
pub mod polyrel;
pub mod synthgo;

pub trait SynthGenerator: Send {
	fn set_sr(&mut self, sr: usize);
	fn generate(&mut self, note: u8, velocity: f32) -> Box<dyn Synth>;
}

pub trait Synth: Send {
	// after smp_count samples, the key will be up
	// will not be called more than once
	fn set_end(&mut self, smp_count: usize);

	// data contains N frames, the synth state will also go forward for N frames
	// return Some(frame) if finished in N frame(so it is frame perfect)
	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> bool;
}
