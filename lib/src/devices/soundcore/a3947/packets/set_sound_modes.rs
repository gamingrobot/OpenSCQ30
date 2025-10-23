use crate::devices::soundcore::{
    a3947::structures::A3947SoundModes,
    common::packet::{self, Command, outbound::ToPacket},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct A3947SetSoundModesPacket {
    pub sound_modes: A3947SoundModes,
}

impl ToPacket for A3947SetSoundModesPacket {
    type DirectionMarker = packet::OutboundMarker;

    fn command(&self) -> Command {
        Command([0x06, 0x81])
    }

    fn body(&self) -> Vec<u8> {
        self.sound_modes.bytes().to_vec()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::devices::soundcore::{
//         a3947::{
//             packets::set_sound_modes::A3947SetSoundModesPacket,
//             structures::{
//                 A3947NoiseCancelingMode, A3947SoundModes, AdaptiveNoiseCanceling,
//                 ManualNoiseCanceling, WindNoise,
//             },
//         },
//         common::{
//             packet::outbound::ToPacket,
//             structures::{AmbientSoundMode, TransparencyMode},
//         },
//     };

//     #[test]
//     fn it_matches_an_example_packet() {
//         const EXPECTED: &[u8] = &[
//             0x08, 0xee, 0x00, 0x00, 0x00, 0x06, 0x81, 0x10, 0x00, 0x02, 0x12, 0x00, 0x01, 0x01,
//             0x02, 0xa5,
//         ];
//         let packet = A3947SetSoundModesPacket {
//             sound_modes: A3947SoundModes {
//                 ambient_sound_mode: AmbientSoundMode::Normal,
//                 manual_noise_canceling: ManualNoiseCanceling::Weak,
//                 adaptive_noise_canceling: AdaptiveNoiseCanceling::HighNoise,
//                 transparency_mode: TransparencyMode::FullyTransparent,
//                 noise_canceling_mode: A3947NoiseCancelingMode::Manual,
//                 wind_noise: WindNoise {
//                     is_detected: false,
//                     is_suppression_enabled: true,
//                 },
//                 noise_canceling_adaptive_sensitivity_level: 2,
//             },
//         };
//         assert_eq!(EXPECTED, packet.to_packet().bytes());
//     }
// }
