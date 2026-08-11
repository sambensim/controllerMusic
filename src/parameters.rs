pub struct ParamInfo {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

pub trait Parameterized {
    fn params(&self) -> &'static [ParamInfo];

    fn set_param(&mut self, index : usize, value : f32);
}

#[macro_export]
macro_rules! params { //AI
    ($ty:ty { $($name:literal => $field:ident [$min:expr, $max:expr, $default:expr]),* $(,)? }) => {
        impl $ty {
            pub const PARAMS: &'static [$crate::parameters::ParamInfo] = &[
                $($crate::parameters::ParamInfo {
                    name: $name, min: $min, max: $max, default: $default
                },)*
            ];
        }

        impl $crate::parameters::Parameterized for $ty {
            fn params(&self) -> &'static [$crate::parameters::ParamInfo] {
                Self::PARAMS
            }

            #[allow(unused_mut, unused_assignments)]
            fn set_param(&mut self, index: usize, value: f32) {
                let mut i = 0usize;
                $(
                    if index == i { self.$field = value; return; }
                    i += 1;
                )*
                debug_assert!(false, "param index {} out of range", index);
            }
        }
    };
}