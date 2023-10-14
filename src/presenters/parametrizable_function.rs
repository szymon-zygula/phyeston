use crate::numerics::{Float, FloatFn};
use egui::Ui;

pub trait ParametrizableFunction {
    type F: Float;

    fn name(&self) -> &str;

    /// `true` on change, `false` otherwise
    fn manipulation_ui(&mut self, ui: &mut Ui) -> bool;

    fn produce_closure(&self) -> FloatFn<Self::F>;
}

macro_rules! parametrizable_function {
    ($mod_name:ident::$struct_name:ident($name:expr => |$arg:ident| $formula:expr): $( $var_param:ident | $var_param_name:expr),*) => {
        pub mod $mod_name {
            pub struct Ranges<F: crate::numerics::Float> {
                $($var_param: std::ops::RangeInclusive<F>,)*
            }

            impl<F: crate::numerics::Float> Ranges<F> {
                pub fn new($($var_param: std::ops::RangeInclusive<F>,)*) -> Self {
                    Self {
                        $($var_param,)*
                    }
                }
            }

            pub struct $struct_name<F: crate::numerics::Float> {
                $($var_param: F,)*
                ranges: Ranges<F>,
            }

            impl<F: crate::numerics::Float> $struct_name<F> {
                pub fn new(
                    $($var_param: F,)*
                    ranges: Ranges<F>
                ) -> Self {
                    Self {
                        $($var_param,)*
                        ranges
                    }
                }
            }

            impl<F: crate::numerics::Float>
                crate::presenters::parametrizable_function::ParametrizableFunction
                for $struct_name<F> {
                type F = F;

                fn name(&self) -> &str {
                    $name
                }

                fn manipulation_ui(&mut self, ui: &mut egui::Ui) -> bool {
                    // | instead of || to avoid short-circuiting
                    false
                    $(
                        | ui.add(
                            egui::widgets::Slider::new(&mut self.$var_param, self.ranges.$var_param.clone())
                            .text($var_param_name)
                        ).changed()
                    )*
                }

                fn produce_closure(&self) -> crate::numerics::FloatFn<Self::F> {
                    $(
                        let $var_param = self.$var_param;
                    )*
                    Box::new(move |$arg| {$formula})
                }
            }
        }

        pub use $mod_name::$struct_name;
    };
}

parametrizable_function!(constant_function::ConstantFunction("A" => |_t| value): value | "A");

parametrizable_function!(
    step_function::StepFunction(
        "t<0=>0, t>0=>A" => |t| {
        if t < t_0 {
            F::zero()
        }
        else {
            amplitude
        }
    }):
    amplitude | "A",
    t_0 | "t_0"
);

parametrizable_function!(
    step_sine::StepSine(
    "A*sgn(sin(ω*t+φ))" => |t|
        if num_traits::Float::sin(t * frequency + phase) > F::zero() {
            amplitude
        }
        else {
            -amplitude
        }
    ):
    amplitude | "A",
    frequency | "ω",
    phase | "φ"
);

parametrizable_function!(
    sine::Sine(
    "A*sin(ω*t+φ)" => |t|
        amplitude * num_traits::Float::sin(t * frequency + phase)):
    amplitude | "A",
    frequency | "ω",
    phase | "φ"
);
