use polysplit::{poly_host::PolyHost, Synth, SynthGenerator};
use std::f32::consts::PI;

struct SineSynth {
	end: Option<usize>,
	f: f32,
	phase: f32,
	si: f32,
	v: f32,
}

impl Synth for SineSynth {
	fn set_end(&mut self, smp_count: usize) {
		self.end = Some(smp_count);
	}
	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> bool {
		for (idx, (sl, sr)) in data_l.iter_mut().zip(data_r.iter_mut()).enumerate() {
			let dphase = self.f * self.si;
			self.phase += dphase;
			let v = self.phase.sin() * self.v * 0.5;
			*sl += v;
			*sr += v;
			if let Some(e) = self.end {
				if e >= idx {
					return true
				}
			}
			while self.phase >= PI * 2.0 {
				self.phase -= PI * 2.0;
			}
		}
		return false
	}
}

#[derive(Default)]
struct SineGen {
	sr: usize,
}
impl SynthGenerator for SineGen {
	fn set_sr(&mut self, sr: usize) {
		self.sr = sr;
	}
	fn generate(&mut self, freq: f32, velocity: f32) -> Box<dyn Synth> {
		eprintln!("{} {}", freq, velocity);
		let ss = SineSynth {
			end: None,
			f: freq,
			phase: 0f32,
			si: 2.0 * PI / self.sr as f32,
			v: velocity,
		};
		Box::new(ss)
	}
}

fn main() {
	let ph = PolyHost::new(Box::new(SineGen::default()));
	ph.run();
}
