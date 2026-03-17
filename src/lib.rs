use core::f64;
use std::{f32::consts::PI, num::NonZeroU32, sync::{Arc, Mutex}};
use nih_plug::{buffer::ChannelSamples, params::persist, plugin::Plugin, prelude::*};
use nih_plug_vizia::ViziaState;

mod editor;

const NUM_CHANNELS: u32 = 2;
const MAX_BLOCK_SIZE: usize = 64;

struct Plugged {
    params: Arc<PluggedParams>,
}

#[derive(Params)]
struct PluggedParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[id = "dry_mix"]
    mix: FloatParam,
    #[id = "drive"]
    drive: FloatParam,
    #[persist = "debug"]
    diff: Mutex<Vec<f32>>,
}

impl Default for Plugged {
    fn default() -> Self {
        Self { params: Arc::new(PluggedParams::default()), }
    }
}

impl Default for PluggedParams {
    fn default()  -> Self {
        Self {
            editor_state: editor::default_state(),
            mix: FloatParam::new("mix", 0.0, FloatRange::Linear {
                min: 0.0,
                max: 1.0
            }),
            drive: FloatParam::new("drive", 0.0, FloatRange::Linear { min: 0.0, max: 5.0 }),
            diff: Mutex::new(vec![]),
        }
    }
}

impl Plugin for Plugged {
    const NAME: &'static str = "The First Plug";

    const VENDOR: &'static str = "Moist Pyro";

    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");

    const EMAIL: &'static str = "suckma@ass.sx";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(NUM_CHANNELS),
        main_output_channels: NonZeroU32::new(NUM_CHANNELS),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
            &mut self,
            audio_io_layout: &AudioIOLayout,
            buffer_config: &BufferConfig,
            context: &mut impl InitContext<Self>,
        ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for (_, mut block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            for samples in block.iter_samples() {

                let wet = self.params.mix.smoothed.next();
                let dry = 1.0 - wet;
                let drive = self.params.drive.smoothed.next();
                let mut diff = self.params.diff.lock().unwrap();

                let mut diffs = vec![];
                
                for sample in samples {
                    let temp = *sample;
                    *sample = Self::wierdify(*sample, drive) * wet + *sample * dry;
                    diffs.push(*sample - temp);
                }
                diffs.sort_by(|a, b| ((a * 100.0) as i64).cmp(&((b * 100.0) as i64)));
                *diff = diffs;
            }
        }
        
        ProcessStatus::Normal
    }
}

impl Plugged {
    fn wierdify(sample: f32, drive: f32) -> f32 {
        let distortion = |x: f32| x / x.abs() * (1.0 - (-1.0 * drive * x * x / x.abs()).exp());
        let error_correct = (1.0 - distortion(1.0)) * sample;
        distortion(sample) + error_correct
    }
}

impl Vst3Plugin for Plugged {
    const VST3_CLASS_ID: [u8; 16] = *b"First_Pluggedabc";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
    ];
}

impl ClapPlugin for Plugged {
    const CLAP_ID: &'static str = "first_plugged";

    const CLAP_DESCRIPTION: Option<&'static str> = Some("first test pluggin. no features guarantied.");

    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
    ];
}

nih_export_vst3!(Plugged);
nih_export_clap!(Plugged);