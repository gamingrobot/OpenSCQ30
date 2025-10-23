use nom::{
    IResult, Parser,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use openscq30_i18n_macros::Translate;
use strum::{Display, EnumIter, EnumString, FromRepr, IntoStaticStr};

use crate::devices::soundcore::common::{packet::parsing::take_bool, structures::{AmbientSoundMode, TransparencyMode}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct A3947SoundModes {
    pub ambient_sound_mode: AmbientSoundMode,
    pub transparency_mode: TransparencyMode,
    pub manual_noise_canceling: u8,
    pub noise_canceling_mode: A3947NoiseCancelingMode,
    pub wind_noise: WindNoise,
    pub adaptive_env_detection:  bool,
    pub transport_noise_canceling: TransportNoiseCanceling,
}

impl A3947SoundModes {
    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3947 sound modes",
            map(
                (
                    AmbientSoundMode::take,
                    le_u8, //manual mode level
                    TransparencyMode::take,
                    A3947NoiseCancelingMode::take,
                    WindNoise::take,
                    take_bool, //env detection
                    TransportNoiseCanceling::take,
                ),
                |(
                    ambient_sound_mode,
                    manual_noise_canceling,
                    transparency_mode,
                    noise_canceling_mode,
                    wind_noise,
                    adaptive_env_detection,
                    transport_noise_canceling
                )| {
                    Self {
                        ambient_sound_mode,
                        manual_noise_canceling,
                        transparency_mode,
                        noise_canceling_mode,
                        wind_noise,
                        adaptive_env_detection,
                        transport_noise_canceling
                    }
                },
            ),
        )
        .parse_complete(input)
    }

    pub(crate) fn bytes(&self) -> [u8; 7] {
        [
            self.ambient_sound_mode.id(),
            self.manual_noise_canceling,
            self.transparency_mode.id(),
            self.noise_canceling_mode.id(),
            self.wind_noise.byte(),
            self.adaptive_env_detection as u8,
            self.transport_noise_canceling.id()
        ]
    }
}

#[repr(u8)]
#[derive(
    FromRepr,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Display,
    Default,
    IntoStaticStr,
    EnumString,
    EnumIter,
    Translate,
)]
pub enum TransportNoiseCanceling {
    #[default]
    Plane = 0,
    Train = 1,
    Bus = 2,
    Car = 3,
}

impl TransportNoiseCanceling {
    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "transportation mode",
            map(le_u8, |transport_noise_canceling| {
                Self::from_repr(transport_noise_canceling).unwrap_or_default()
            }),
        )
        .parse_complete(input)
    }
}

impl TransportNoiseCanceling {
    pub fn id(&self) -> u8 {
        *self as u8
    }
}



#[repr(u8)]
#[derive(
    FromRepr,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Display,
    Default,
    IntoStaticStr,
    EnumString,
    EnumIter,
    Translate,
)]
pub enum A3947NoiseCancelingMode {
    #[default]
    Manual = 0,
    Adaptive = 1,
    Transport = 2,
}

impl A3947NoiseCancelingMode {
    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3947 noise canceling mode",
            map(le_u8, |noise_canceling_mode| {
                Self::from_repr(noise_canceling_mode).unwrap_or_default()
            }),
        )
        .parse_complete(input)
    }
}

impl A3947NoiseCancelingMode {
    pub fn id(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindNoise {
    pub is_suppression_enabled: bool,
    pub is_detected: bool,
}

impl WindNoise {
    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "wind noise",
            map(le_u8, |wind_noise| Self {
                is_suppression_enabled: wind_noise & 1 != 0,
                is_detected: wind_noise & 2 != 0,
            }),
        )
        .parse_complete(input)
    }

    pub fn byte(&self) -> u8 {
        u8::from(self.is_suppression_enabled) | (u8::from(self.is_detected) << 1)
    }
}
