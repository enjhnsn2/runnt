use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ActivationType {
    Relu,    // max(0,val)
    Sigmoid, // 0 to 1
    Linear,  // val
    Tanh,    //-1 to 1
}

pub fn activate(val: f32, ltype: ActivationType) -> f32 {
    match ltype {
        ActivationType::Relu => val.max(0.),
        ActivationType::Sigmoid => 1. / (1. + (-val).exp()),
        ActivationType::Linear => val,
        ActivationType::Tanh => val.tanh(),
    }
}
///Note this is on the activated value, so we dont need to activate it again
pub fn activate_der(activated_value: f32, ltype: ActivationType) -> f32 {
    match ltype {
        ActivationType::Relu => {
            if activated_value > 0. {
                1.
            } else {
                0.
            }
        }
        ActivationType::Sigmoid => activated_value * (1. - activated_value),
        ActivationType::Linear => 1.,
        ActivationType::Tanh => 1. - activated_value * activated_value, //1-tanh(x)^2 (since we have val=tanh(x), we just use val)
    }
}

impl Display for ActivationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivationType::Relu => write!(f, "Relu"),
            ActivationType::Sigmoid => write!(f, "Sigmoid"),
            ActivationType::Linear => write!(f, "Linear"),
            ActivationType::Tanh => write!(f, "Tanh"),
        }
    }
}

impl FromStr for ActivationType {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Relu" => Ok(ActivationType::Relu),
            "Sigmoid" => Ok(ActivationType::Sigmoid),
            "Linear" => Ok(ActivationType::Linear),
            "Tanh" => Ok(ActivationType::Tanh),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unknown activation type",
            )),
        }
    }
}
