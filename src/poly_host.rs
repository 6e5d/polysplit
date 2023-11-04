use crate::{Synth, SynthGenerator};

use std::collections::HashMap;

pub struct PolyHost {
	// TODO: synth generator generate() cost too much(many components to build)
	generator: Box<dyn SynthGenerator>,
	active: HashMap<u8, Box<dyn Synth>>,
	sustain_flag: bool,
	sustain: Vec<Box<dyn Synth>>,
	release: Vec<Box<dyn Synth>>,
}

impl PolyHost {
	pub fn new(generator: Box<dyn SynthGenerator>) -> Self {
		Self {
			generator,
			active: Default::default(),
			sustain_flag: false,
			sustain: Default::default(),
			release: Default::default(),
		}
	}

	pub fn run(mut self) {
		let (client, _status) = jack::Client::new(
			"polysplit",
			jack::ClientOptions::NO_START_SERVER,
		).unwrap();
		let sample_rate = client.sample_rate();
		self.generator.set_sr(sample_rate);

		let midi_in = client
			.register_port("midi_in", jack::MidiIn::default())
			.unwrap();
		let mut audio_out1 = client
			.register_port("audio_out1", jack::AudioOut::default())
			.unwrap();
		let mut audio_out2 = client
			.register_port("audio_out2", jack::AudioOut::default())
			.unwrap();

		let callback =
			move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
				for event in midi_in.iter(ps) {
					if event.bytes[0] & 0xf0 == 0x90 {
						let note = event.bytes[1];
						let synth = self.generator.generate(
							note,
							event.bytes[2] as f32 / 128.0,
						);
						if let Some(_) = self.active.insert(event.bytes[1], synth) {
							eprintln!("ERROR, missing keyup event {:?}", event.bytes[1]);
						}
					} else if event.bytes[0] & 0xf0 == 0x80 {
						// NOTE: directly overwrite or sort earliest release event?
						if let Some(mut synth) = self.active.remove(&event.bytes[1]) {
							if self.sustain_flag {
								self.sustain.push(synth);
							} else {
								synth.set_end(event.time as usize);
								self.release.push(synth);
							}
						}
					} else if event.bytes[0] & 0xf0 == 0xb0 && event.bytes[1] == 64 {
						if event.bytes[2] == 127 {
							self.sustain_flag = false;
							for mut synth in std::mem::take(&mut self.sustain).into_iter() {
								synth.set_end(event.time as usize);
								self.release.push(synth);
							}
						} else if event.bytes[2] == 0 {
							self.sustain_flag = true;
						}
					} else {
						eprintln!("unhandled {:?}", event.bytes);
					}
				}
				let out1 = audio_out1.as_mut_slice(ps);
				let out2 = audio_out2.as_mut_slice(ps);
				for v in out1.iter_mut() { *v = 0.0 }
				for v in out2.iter_mut() { *v = 0.0 }
				for (key, mut synth) in std::mem::take(&mut self.active).into_iter() {
					if !synth.sample(out1, out2) {
						self.active.insert(key, synth);
					}
				}
				for mut synth in std::mem::take(&mut self.sustain).into_iter() {
					if !synth.sample(out1, out2) {
						self.sustain.push(synth);
					}
				}
				for mut synth in std::mem::take(&mut self.release).into_iter() {
					if !synth.sample(out1, out2) {
						self.release.push(synth);
					}
				}
				jack::Control::Continue
			};

		let active_client = client
			.activate_async((), jack::ClosureProcessHandler::new(callback))
			.unwrap();
		std::thread::park();
		active_client.deactivate().unwrap();
	}
}
