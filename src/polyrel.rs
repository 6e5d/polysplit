use crate::Synth;

pub trait Polyrel: Send {
	fn go(&mut self) -> Option<[f32; 2]>;
}

pub struct PolyrelWrapper {
	synth: Box<dyn Polyrel>,
	rk: f32,
	release_len: f32,
	release_idx: i32,
	end: Option<usize>,
}

impl PolyrelWrapper {
	pub fn new(synth: Box<dyn Polyrel>, rk: f32, release_len: f32) -> Self {
		Self {
			synth,
			// amplitude: (1 - x)^rk
			rk,
			// total release length(in sample count)
			release_len,
			// current index after release start
			release_idx: -1,
			end: None,
		}
	}
}

impl Synth for PolyrelWrapper {
	fn set_end(&mut self, smp_count: usize) { self.end = Some(smp_count); }
	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> bool {
		for (idx, (sl, sr)) in data_l.iter_mut().zip(data_r.iter_mut()).enumerate() {
			let mut v = match self.synth.go() {
				None => return true,
				Some(v) => v,
			};
			if self.release_idx >= 0 {
				let x = (1f32 - (self.release_idx as f32 / self.release_len)).powf(self.rk);
				self.release_idx += 1;
				if self.release_idx as f32 >= self.release_len { return true }
				v[0] *= x;
				v[1] *= x;
			} else {
				if let Some(e) = self.end {
					if e >= idx {
						self.release_idx = 0;
					}
				}
			}
			*sl += v[0];
			*sr += v[1];
		}
		return false
	}
}
