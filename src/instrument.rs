use core::panic;

use crate::{adsr::Adsr, controller::{InputEvent, InputSource}, effect::Effect, oscillator::Oscillator, voice_manager::{VoiceManager, Voicebank}};

#[derive(PartialEq)]
pub struct ParamInfo {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

pub struct ParamOverride {
    pub target: TargetSpec,
    pub param: &'static str,
    pub value: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum TargetSpec { Effect(&'static str), Osc }

#[derive(Clone, Copy, Debug)]
pub enum Target { Effect(usize), Osc }

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Curve { Linear(), Exponential(f32), Stepped(u8)}

const fn map(value : f32, min_in : f32, max_in : f32, min_out : f32, max_out : f32) -> f32 {
    (value - min_in) / max_in * max_out + min_out
}

const fn discretize(value : f32, min : f32, max : f32, steps : u8) -> f32 {
    map(((map(value, min, max, 0.0, 1.0) * steps as f32) as u8) as f32, 0.0, steps as f32, min, max)
}

impl Curve {
    fn apply(&self, value : f32, info : &ParamInfo) -> f32 {
        // println!("{} -> [{} {}] = {}", value, info.min, info.max, map(value, 0.0, 1.0, info.min, info.max));
        map(
            match self {
                Curve::Linear() => value,
                Curve::Exponential(exp) => value.powf(*exp),
                Curve::Stepped(steps) => discretize(value, 0.0, 1.0, *steps),
            },
        0.0, 1.0, info.min, info.max
        )
    }
}

const MAX_EFFECTS : usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct InputMapSpec {
    pub target : TargetSpec,
    pub param : &'static str,
    pub input : InputSource,
    pub response : Curve,
}

impl InputMapSpec {
    fn bake(self, effects : &Vec<(&'static str, Box<dyn Effect>)>, osc_params : &'static [ParamInfo]) -> InputMapping {
        let mut effect_index = MAX_EFFECTS;
        let mut param_index = MAX_EFFECTS;
        if let TargetSpec::Effect(effect_name) = self.target {
            for (i, p) in effects.into_iter().enumerate() {
                if p.0 == effect_name {
                   if let Some(index) = p.1.params().iter().position(|param| param.name == self.param) {
                        param_index = index;
                        effect_index = i;
                        break;
                    }
                }
            }
            if param_index == MAX_EFFECTS {
                panic!("bad effect param '{}' for effect '{}'", self.param, effect_name)
            }
        } else {
            if let Some(p_i) = osc_params.iter().position(|param| param.name == self.param) {
                param_index = p_i
            } else {
                panic!("bad osc param '{}'", self.param)
            }
        }
        InputMapping {
            target_index: effect_index,
            param_index: param_index,
            input: self.input,
            response: self.response
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InputMapping {
    target_index : usize,
    param_index : usize,
    input : InputSource,
    response : Curve,
}

impl InputMapping {
    pub fn try_input(&self, effects : &mut Vec<Box<dyn Effect>>, voicebank : &mut Voicebank, event : InputEvent) {
        let (e, v) = event.split();
        if e == self.input {
            if self.target_index == MAX_EFFECTS {
                voicebank.set(self.param_index, self.response.apply(v, &voicebank.params()[self.param_index]));
            } else {
                let info = &effects[self.target_index].params()[self.param_index];
                effects[self.target_index].set_param(self.param_index, self.response.apply(v, info));
            }
        }
    }
}

pub struct Instrument {
    display_name : &'static str,
    voicebank : Voicebank,
    post_processing : Vec<Box<dyn Effect>>,
    input_map : Vec<InputMapping>
}

impl Instrument {
    pub fn new(
        sample_rate: u32, display_name : &'static str,
        oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>,
        envelope_factory: impl Fn(u32) -> Adsr,
        voice_manager : &mut VoiceManager,
        effects : &mut Vec<(&'static str, Box<dyn Effect>)>,
        input_map : &mut Vec<InputMapSpec>,
        overrides : Vec<ParamOverride>,
        polyphony : usize) -> Self
    {
        let osc_overrides = Self::apply_overrides(overrides, effects);
        let voicebank = voice_manager.request_voicebank(sample_rate, polyphony, &oscillator_factory, &envelope_factory).unwrap();
        for o in osc_overrides {
            let _ = voicebank.params().iter().position(|p| p.name == o.param).unwrap();
        }
        
        let input_map_baked : Vec<InputMapping> = input_map.iter_mut().map(| spec | {spec.bake(&effects, voicebank.params())}).collect();
        let post_processing = effects.drain(..).map(|(_, e)| e).collect();
        Instrument {
            display_name : display_name,
            voicebank: voicebank,
            post_processing: post_processing,
            input_map : input_map_baked,
        }
    }

    fn apply_overrides(overrides : Vec<ParamOverride>, effects : &mut [(&'static str, Box<dyn Effect>)]) -> Vec<ParamOverride> {
        let mut osc_overrides : Vec<ParamOverride> = vec![];
        for o in overrides {
            match o.target {
                TargetSpec::Effect(name) => {
                    let (_, effect) = effects.iter_mut().find(|(label, _)| *label == name).unwrap();
                    let index = effect.params().iter().position(|p| p.name == o.param).unwrap();
                    effect.set_param(index, o.value);
                },
                TargetSpec::Osc => osc_overrides.push(o),
            }
        }
        osc_overrides
    }

    fn seed_defaults(fx: &mut dyn Effect) {
        for (i, info) in fx.params().iter().enumerate() {
            fx.set_param(i, info.default);
        }
    }

    pub fn step(&mut self) -> f32 {
        let mut out = self.voicebank.step();
        for e in &mut self.post_processing {
            out = e.step(out);
        }
        out
    }

    pub fn handle_input(&mut self, event : &InputEvent) {
        for route in self.input_map.iter() {
            route.try_input(&mut self.post_processing, &mut self.voicebank, *event);
        }
    }

    pub fn play(&mut self, note : u8) {
        self.voicebank.play(note);
    }

    pub fn release(&mut self, note : u8) {
        self.voicebank.release(note);
    }

    pub fn release_all(&mut self) {
        self.voicebank.release_all();
    }
}