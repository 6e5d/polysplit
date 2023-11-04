use crate::Synth;

pub trait Synthgo: Send {
	fn go(&mut self) -> Option<[f32; 2]>;
	fn keyup(&mut self);
}

pub struct SynthgoWrapper {
	synth: Box<dyn Synthgo>,
	end: Option<usize>,
}

impl SynthgoWrapper {
	pub fn new(synth: Box<dyn Synthgo>) -> Self {
		Self {
			synth,
			end: None,
		}
	}
}

impl Synth for SynthgoWrapper {
	fn set_end(&mut self, smp_count: usize) { self.end = Some(smp_count); }
	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> bool {
		for (idx, (sl, sr)) in data_l.iter_mut().zip(data_r.iter_mut()).enumerate() {
			let v = match self.synth.go() {
				None => return true,
				Some(v) => v,
			};
			if let Some(e) = self.end {
				if e >= idx {
					self.synth.keyup();
				}
			}
			*sl += v[0];
			*sr += v[1];
		}
		return false
	}
}
