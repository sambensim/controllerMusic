use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
    Device, Error, ErrorKind, FromSample, OutputCallbackInfo, Sample, SampleFormat,
    SizedSample, StreamConfig, I24,
};
use std::collections::VecDeque;


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

fn sound_main(sample_rate: f32, audio: impl Fn(f32, f32, f32) -> f32 + Send + 'static) -> (impl FnMut() -> f32 + Send + 'static, impl FnMut(&mut std::collections::VecDeque<f32>)) {
    use ringbuf::{HeapRb, traits::{Split, Producer, Consumer}};
    const DISPLAY_SAMPLES: usize = 1024; // how many samples to show at once
    let mut time = 0.0f32;
    let mut absolute_time = 0.0f32;  // never wraps

    let rb = HeapRb::<f32>::new(4096); // big enough not to overflow between frames
    let (mut producer, mut consumer) = rb.split();

    let audio_closure = move || {
        time = (time + 1.0) % sample_rate;
        absolute_time += 1.0;
        let value = audio(time, absolute_time, sample_rate);
        // let value = (time * PITCH * 2.0 * std::f32::consts::PI / sample_rate).sin();
        let _ = producer.try_push(value); // drop samples if visual side is behind
        value
    };

    let visual_closure = move |display_buffer: &mut VecDeque<f32>| {
        while let Some(sample) = consumer.try_pop() {
            display_buffer.push_back(sample);
            if display_buffer.len() > DISPLAY_SAMPLES {
                display_buffer.pop_front();
            }
        }
    };

    (audio_closure, visual_closure)
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

use std::sync::{Arc, Mutex};

use crate::controller::DS4State;

pub fn do_sound(controller_state : Arc<Mutex<DS4State>>) -> impl FnMut(&mut VecDeque<f32>) {
    let (_host, _input_device, _input_config, output_device, output_config) = set_defaults(true);
    let sample_rate = output_config.sample_rate() as f32;

    let cond_sine = {
        let controller_state = Arc::clone(&controller_state);
        move |time: f32, _: f32, sample_rate: f32| -> f32 {
            const PITCH: f32 = 440.0;
            let state = controller_state.lock().unwrap();
            if !state.circle {
                return 0.0;
            }
            let coefficient = 2.0 * std::f32::consts::PI / sample_rate;
            (time * PITCH * coefficient).sin()
        }
    };

    fn sine(time: f32, _ : f32, sample_rate: f32) -> f32 {
        const PITCH: f32 = 440.0;
        let coefficient = 2.0 * std::f32::consts::PI / sample_rate;
        (time * PITCH * coefficient).sin()
    }

    fn demo(time: f32, absolute_time : f32, sample_rate: f32) -> f32 {
        fn make_sine(freq: f32, volume: f32) -> impl Fn(f32, f32, f32) -> f32 {
            move |time, _ : f32, sample_rate| {
                let coefficient = 2.0 * std::f32::consts::PI / sample_rate;
                (time * freq * coefficient).sin() * volume
            }
        }
        match (absolute_time / (sample_rate * 1.0)).floor() % 5.0 {
            0.0 => {
                return make_sine(220.0, 1.0)(time, absolute_time, sample_rate);
            }
            1.0 => {
                return make_sine(440.0, 0.6)(time, absolute_time, sample_rate);
            }
            2.0 => {
                return make_sine(880.0, 0.8)(time, absolute_time, sample_rate);
            }
            3.0 => {
                return make_sine(660.0, 0.8)(time, absolute_time, sample_rate);
            }
            4.0 => {
                return make_sine(440.0, 0.5)(time, absolute_time, sample_rate) + make_sine(660.0, 0.5)(time, absolute_time, sample_rate);
            }
            _ => {
                panic!()
            }
        }
    }

    fn square(time: f32, _ : f32, sample_rate: f32) -> f32 {
        const PITCH: f32 = 440.0;
        if (time * PITCH / sample_rate).fract() < 0.5 { 1.0 } else { -1.0 }
    }

    let (next_value, update_visual) = sound_main(sample_rate, cond_sine);

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
    return update_visual
}