use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
    Device, Error, ErrorKind, FromSample, OutputCallbackInfo, Sample, SampleFormat,
    SizedSample, StreamConfig, I24,
};
use raylib::audio::Sound;
use std::{collections::VecDeque, sync::mpsc::{self, Receiver}};


fn print_input_options(host : &cpal::Host) {
    println!("available input options:");
    let input_options = host.input_devices().unwrap();
    input_options.for_each(|inopt| { println!("\t- {}", inopt) });
}

fn print_output_options(host : &cpal::Host) {
    println!("available output options:");
    let output_options = host.output_devices().unwrap();
    output_options.for_each(|outopt| { println!("\t- {}", outopt) });
}

fn print_input_configs(device : &cpal::Device) {
    println!("available input configs for {}:", device);
    let configs = device.supported_input_configs().unwrap();
    configs.for_each(|config| { println!("\t- {:?}", config)})
}

fn print_output_configs(device : &cpal::Device) {
    println!("available output configs for {}:", device);
    let configs = device.supported_output_configs().unwrap();
    configs.for_each(|config| { println!("\t- {:?}", config)})
}

fn set_defaults(silent : bool) -> (cpal::Host, Device, cpal::SupportedStreamConfig, Device, cpal::SupportedStreamConfig) {
    let host = cpal::default_host();

    let input_device = host.default_input_device().unwrap();
    let input_config = input_device.supported_input_configs().unwrap().next().unwrap().with_max_sample_rate();
    
    let output_device = host.default_output_device().unwrap();
    let output_config = output_device.supported_output_configs().unwrap().next().unwrap().with_max_sample_rate();

    if !silent {
        print_input_options(&host);
        println!("using default input device ({})", input_device);
        print_input_configs(&input_device);
        println!("using default input config ({:?})", input_config);

        print_output_options(&host);
        println!("using default output device ({})", output_device);
        print_output_configs(&output_device);
        println!("using default output config ({:?})", output_config);
    }

    return (host, input_device, input_config, output_device, output_config)
}

type SynthFn = fn(time: f32, absolute_time : f32, sample_rate: f32) -> f32;

fn sound_main(sample_rate: f32, mut audio: impl FnMut(f32, f32, f32) -> f32 + Send + 'static) -> impl FnMut() -> f32 + Send + 'static {
    use ringbuf::{HeapRb, traits::{Split, Producer, Consumer}};
    let mut time = 0.0f32;
    let mut absolute_time = 0.0f32;  // never wraps

    let rb = HeapRb::<f32>::new(4096); // big enough not to overflow between frames
    let (mut producer, mut consumer) = rb.split();

    let audio_closure = move || {
        time = (time + 1.0) % sample_rate;
        absolute_time += 1.0;
        let value = audio(time, absolute_time, sample_rate);
        let _ = producer.try_push(value); // drop samples if visual side is behind
        value
    };

    audio_closure
}

//below was stolen from the examples
fn run<T>(device: &Device, config: StreamConfig, mut value_callback : impl FnMut() -> f32 + std::marker::Send + 'static)
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    
    let err_fn = |err: Error| match err.kind() {
        ErrorKind::DeviceChanged | ErrorKind::Xrun | ErrorKind::RealtimeDenied => {
            eprintln!("{err}")
        }
        _ => eprintln!("Stream error: {err}"),
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &OutputCallbackInfo| write_data(data, channels, &mut value_callback),
        err_fn,
        None,
    ).unwrap();
    stream.play().unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

use crate::{controller::{self, DS4State}, music_theory::ChordEngine};
use crate::music_theory;

struct SoundEngine {
    controller_state : DS4State,
    controller_channel : Receiver<DS4State>,
    chord_engine : music_theory::ChordEngine,
    phases : [f32; SoundEngine::MAX_NOTES],
    freq_send : mpsc::Sender<f32>,
}

impl SoundEngine {
    fn get_state(&mut self) -> DS4State {
        let new_state = self.controller_channel.try_recv();
        if !new_state.is_err() {
            self.controller_state = new_state.unwrap();
        };
        self.controller_state
    }

    fn get_chord(&mut self) -> Vec<f32> {
        let state = self.get_state();
        let oct = controller::get_left_stick_section(&state);
        if oct == -1 {
            return vec!();
        };
        let notes = self.chord_engine.get_chord_notes(oct as i32, 0);
        notes.iter().map(|n : &String| ChordEngine::note_to_freq(n).unwrap()).collect()
    }

    const MAX_NOTES : usize = 4;
}


pub fn do_sound(controller_channel : Receiver<DS4State>) -> Receiver<f32> {
    let (_host, _input_device, _input_config, output_device, output_config) = set_defaults(true);
    let sample_rate = output_config.sample_rate() as f32;
    
    let (sender, receiver) = mpsc::channel();

    let mut sound_engine = SoundEngine {
        controller_state : controller_channel.recv().unwrap(),
        controller_channel : controller_channel,
        chord_engine : music_theory::ChordEngine::new(0, 4),
        phases : [0.0; SoundEngine::MAX_NOTES],
        freq_send : sender
    };
    
    let coefficient = 1.0 / sample_rate;
    let sound_process = {
        let cb = move |_: f32, _: f32, _: f32| -> f32 {
            let freqs = sound_engine.get_chord();
            let mut out : f32 = 0.0;
            for i in 0..SoundEngine::MAX_NOTES {
                if freqs.get(i).is_none() {
                    sound_engine.phases[i] = 0.0
                } else {
                    sound_engine.phases[i] += freqs[i] * coefficient;
                    sound_engine.phases[i] %= 1.0;
                }
                out += (sound_engine.phases[i] * 2.0 * std::f32::consts::PI).sin();
            };
            out /= freqs.len().max(1) as f32;
            let _ = sound_engine.freq_send.send(out);
            out
        };
        cb
    };


    let next_value = sound_main(sample_rate, sound_process);

    std::thread::spawn(move || {
        match output_config.sample_format() {
            SampleFormat::I8 => run::<i8>(&output_device, output_config.into(),next_value),
            SampleFormat::I16 => run::<i16>(&output_device, output_config.into(),next_value),
            SampleFormat::I24 => run::<I24>(&output_device, output_config.into(),next_value),
            SampleFormat::I32 => run::<i32>(&output_device, output_config.into(),next_value),
            // SampleFormat::I48 => run::<I48>(&device, config.into()),
            SampleFormat::I64 => run::<i64>(&output_device, output_config.into(),next_value),
            SampleFormat::U8 => run::<u8>(&output_device, output_config.into(),next_value),
            SampleFormat::U16 => run::<u16>(&output_device, output_config.into(),next_value),
            // SampleFormat::U24 => run::<U24>(&device, config),
            SampleFormat::U32 => run::<u32>(&output_device, output_config.into(),next_value),
            // SampleFormat::U48 => run::<U48>(&device, config),
            SampleFormat::U64 => run::<u64>(&output_device, output_config.into(),next_value),
            SampleFormat::F32 => run::<f32>(&output_device, output_config.into(),next_value),
            SampleFormat::F64 => run::<f64>(&output_device, output_config.into(),next_value),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
    });
    return receiver
}