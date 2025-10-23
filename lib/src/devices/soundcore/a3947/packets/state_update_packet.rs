use async_trait::async_trait;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::all_consuming,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3947::{self, state::A3947State, structures::A3947SoundModes},
        common::{
            modules::ModuleCollection,
            packet::{
                self, inbound::{FromPacketBody, TryToPacket}, outbound::ToPacket, parsing::take_bool, Command
            },
            packet_manager::PacketHandler,
            structures::{
                button_configuration::ButtonStatusCollection, AgeRange, AmbientSoundModeCycle, AutoPowerOff, BatteryLevel, CustomHearId, DualBattery, DualFirmwareVersion, EqualizerConfiguration, FirmwareVersion, SerialNumber, SingleBattery, TouchTone, TwsStatus, VolumeAdjustments
            },
        },
    },
};

// A3947
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A3947StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub equalizer_configuration: EqualizerConfiguration<2, 10>,
    pub age_range: AgeRange,
    pub custom_hear_id: CustomHearId<2, 10>,
    pub button_configuration: ButtonStatusCollection<6>,
    pub sound_modes: A3947SoundModes,
    pub case_battery: BatteryLevel,
    pub sound_leak_detection: bool,
    pub game_mode_switch: bool,
    pub touch_tone: TouchTone,
    pub ldac: bool,
    pub dual_connection: bool,
    pub limit_volume: bool,
    pub db_limit: u8,
    pub db_refresh_rate: u8,
    pub auto_play_pause: bool,
    pub wearing_tone: bool,
    pub auto_power_off: AutoPowerOff,
    pub touch_lock: bool,
    pub low_battery_tone: bool,
}

impl Default for A3947StateUpdatePacket {
    fn default() -> Self {
        Self {
            tws_status: Default::default(),
            battery: Default::default(),
            dual_firmware_version: Default::default(),
            serial_number: Default::default(),
            equalizer_configuration: EqualizerConfiguration::new_custom_profile([
                VolumeAdjustments::new([0; 10]),
                VolumeAdjustments::new([0; 10]),
            ]),
            age_range: Default::default(),
            custom_hear_id: CustomHearId {
                is_enabled: Default::default(),
                volume_adjustments: [
                    VolumeAdjustments::new([0; 10]),
                    VolumeAdjustments::new([0; 10]),
                ],
                time: Default::default(),
                hear_id_type: Default::default(),
                hear_id_music_type: Default::default(),
                custom_volume_adjustments: Some([
                    VolumeAdjustments::new([0; 10]),
                    VolumeAdjustments::new([0; 10]),
                ]),
            },
            button_configuration: a3947::BUTTON_CONFIGURATION_SETTINGS.default_status_collection(),
            sound_modes: Default::default(),
            case_battery: Default::default(),
            sound_leak_detection: Default::default(),
            game_mode_switch: Default::default(),
            touch_tone: Default::default(),
            ldac: Default::default(),
            dual_connection: Default::default(),
            limit_volume: Default::default(),
            db_limit: Default::default(),
            db_refresh_rate: Default::default(),
            auto_play_pause: Default::default(),
            wearing_tone: Default::default(),
            auto_power_off: Default::default(),
            touch_lock: Default::default(),
            low_battery_tone: Default::default(),
        }
    }
}

impl FromPacketBody for A3947StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3947 state update packet",
            all_consuming(|input| {
                let (input, tws_status) = TwsStatus::take(input)?;
                let (input, battery) = DualBattery::take(input)?;
                let (input, dual_firmware_version) = DualFirmwareVersion::take(input)?;
                let (input, serial_number) = SerialNumber::take(input)?;
                let (input, _unknown_firmware_version) = FirmwareVersion::take(input)?;
                let (input, equalizer_configuration) = EqualizerConfiguration::take(input)?;
                let (input, age_range) = AgeRange::take(input)?;
                let (input, custom_hear_id) = CustomHearId::take_without_music_type(input)?;

                // // For some reason, an offset value is taken before the custom button model, which refers to how many bytes
                // // until the next data to be read. This offset includes the length of the custom button model. Presumably,
                // // there are some extra bytes between the button model and the beginning of the next data to be parsed?
                let (input, skip_offset) = le_u8(input)?;
                let remaining_before_button_configuration = input.len();
                let (input, button_configuration) = ButtonStatusCollection::take(
                    a3947::BUTTON_CONFIGURATION_SETTINGS.parse_settings(),
                )(input)?;
                let button_configuration_size = remaining_before_button_configuration - input.len();
                let (input, _) = take(
                    (skip_offset as usize)
                        // subtract an extra 1 since we want the number of bytes to discard, not
                        // the offset to the first byte to read
                        .checked_sub(button_configuration_size + 2)
                        .unwrap_or_default(),
                )(input)?;

                let (input, _) = le_u8(input)?; //unknown
                let (input, sound_modes) = A3947SoundModes::take(input)?;
                let (input, _) = take(6usize)(input)?; //unknown
                let (input, case_battery) = BatteryLevel::take(input)?;
                let (input, _) = le_u8(input)?; //unknown
                let (input, sound_leak_detection) = take_bool(input)?;
                let (input, _) = le_u8(input)?; //unknown
                let (input, game_mode_switch) = take_bool(input)?;
                let (input, touch_tone) = TouchTone::take(input)?;
                let (input, _) = le_u8(input)?; //unknown
                let (input, ldac) = take_bool(input)?;
                let (input, dual_connection) = take_bool(input)?;
                let (input, _) = le_u8(input)?; //unknown
                let (input, limit_volume) = take_bool(input)?;
                let (input, db_limit) = le_u8(input)?;
                let (input, db_refresh_rate) = le_u8(input)?;
                let (input, auto_play_pause) = take_bool(input)?;
                let (input, wearing_tone) = take_bool(input)?;
                let (input, auto_power_off) = AutoPowerOff::take(input)?;
                let (input, touch_lock) = take_bool(input)?;
                let (input, low_battery_tone) = take_bool(input)?;
                let (input, _) = le_u8(input)?; //end?

                Ok((
                    input,
                    Self {
                        tws_status,
                        battery,
                        dual_firmware_version,
                        serial_number,
                        equalizer_configuration,
                        age_range,
                        custom_hear_id,
                        button_configuration,
                        sound_modes,
                        case_battery,
                        sound_leak_detection,
                        game_mode_switch,
                        touch_tone,
                        ldac,
                        dual_connection,
                        limit_volume,
                        db_limit,
                        db_refresh_rate,
                        auto_play_pause,
                        wearing_tone,
                        auto_power_off,
                        touch_lock,
                        low_battery_tone,
                    },
                ))
            }),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3947StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.tws_status
            .bytes()
            .into_iter()
            .chain(self.battery.bytes())
            .chain(self.dual_firmware_version.bytes())
            .chain(self.serial_number.to_string().into_bytes())
            .chain(self.equalizer_configuration.bytes())
            .chain([self.age_range.0])
            .chain(
                [self.custom_hear_id.is_enabled as u8]
                    .into_iter()
                    .chain(
                        self.custom_hear_id
                            .volume_adjustments
                            .iter()
                            .flat_map(|v| v.bytes()),
                    )
                    .chain(self.custom_hear_id.time.to_le_bytes())
                    .chain([self.custom_hear_id.hear_id_type.0])
                    .chain(
                        self.custom_hear_id
                            .custom_volume_adjustments
                            .as_ref()
                            .unwrap()
                            .iter()
                            .flat_map(|v| v.bytes()),
                    )
                    .chain([0, 0]),
            )
            .chain([0]) // TODO skip offset
            .chain(
                self.button_configuration
                    .bytes(a3947::BUTTON_CONFIGURATION_SETTINGS.parse_settings()),
            )
            .chain([0])
            .chain(self.sound_modes.bytes())
            .chain([0; 6])
            .chain([
                self.case_battery.0,
                self.sound_leak_detection as u8,
                self.game_mode_switch as u8,
                self.touch_tone as u8,
                self.ldac as u8,
                self.dual_connection as u8,
                self.limit_volume as u8,
                self.db_limit,
                self.db_refresh_rate,
                self.auto_play_pause as u8,
                self.wearing_tone as u8,
            ])
            .chain(self.auto_power_off.bytes())
            .chain([
                self.touch_lock as u8,
                self.low_battery_tone as u8,
            ])
            .chain([0])
            .collect()
    }
}

struct StateUpdatePacketHandler {}

#[async_trait]
impl PacketHandler<A3947State> for StateUpdatePacketHandler {
    async fn handle_packet(
        &self,
        state: &watch::Sender<A3947State>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        let packet: A3947StateUpdatePacket = packet.try_to_packet()?;
        state.send_modify(|state| *state = packet.into());
        Ok(())
    }
}

impl ModuleCollection<A3947State> {
    pub fn add_state_update(&mut self) {
        self.packet_handlers.set_handler(
            packet::inbound::STATE_COMMAND,
            Box::new(StateUpdatePacketHandler {}),
        );
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use crate::devices::soundcore::common::packet::inbound::FromPacketBody;

    use super::*;

    #[test]
    fn serialize_and_deserialize() {
        let bytes = A3947StateUpdatePacket::default().to_packet().bytes();
        let (_, packet) = packet::Inbound::take::<VerboseError<_>>(&bytes).unwrap();
        let _: A3947StateUpdatePacket = packet.try_to_packet().unwrap();
    }

    #[test]
    pub fn it_parses_a_known_good_packet() {
        let input = &[
            0x09, 0xFF, 0x00, 0x00, 0x01, 0x01, 0x01, //command
            0xA9, 0x00, //length
            0x01, 0x01, //tws: connected, host
            0x05, 0x05, 0x00, 0x00, //batteryLeft, batteryRight, chargingLeft, chargingRight
            0x30, 0x36, 0x2E, 0x38, 0x38, 0x30, 0x36, 0x2E, 0x38, 0x38, //leftFirmware, rightFirmware
            0x33, 0x39, 0x34, 0x37, 0x37, 0x41, 0x38, 0x44, 0x30, 0x41, 0x38, 0x41, 0x39, 0x44, 0x46, 0x34, //serial
            0x30, 0x30, 0x2E, 0x30, 0x30, //unknown firmware?
            0x15, 0x00, //EQ Preset
            0x78, 0x78, 0x78, 0x78, 0x74, 0x6F, 0x6C, 0x65, 0x78, 0x78, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //EQ
            0x02, //age range?
            0x01, //hearId enabled
            0x90, 0x93, 0x89, 0x89, 0x79, 0x6E, 0x67, 0x64, 0x3C, 0x3C, 0x90, 0x93, 0x89, 0x89, 0x79, 0x6E, 0x67, 0x64, 0x3C, 0x3C, //volumeAdj
            0x67, 0x57, 0x4F, 0x7B, //time
            0x00, // hearId type
            0x79, 0x78, 0x78, 0x79, 0x79, 0x6E, 0x67, 0x64, 0x3C, 0x3C, 0x79, 0x78, 0x78, 0x79, 0x79, 0x6E, 0x67, 0x64, 0x3C, 0x3C, //customVolumeAdj
            0x15, 0x00, // hearId music type?
            0x12, //button len
            0x11, 0x66, 0x11, 0x66, 0x11, 0x32, 0x11, 0x33, 0x11, 0x44, 0x11, 0x44, 0x00, 0xFF, 0x00, 0xFF, //buttons
            0x33, 
            0x02, // ambient mode: 00 noise canceling, 01 transparent, 02 normal
            0x50, // NC manual mode level 10-50
            0x00, // transparency mode: 00 fully transparent, 01 vocal mode
            0x01, // NC mode: 00 manual mode, 01 adaptive, 02 transportation
            0x00, // wind noise reduction
            0x01, // ANC env detection
            0x00, // NC transportation type 00 plane, 01 train, 02 bus, 03 car
            0x00, 0x00, 0x00, 0x00, 0xFF, 0x01,
            0x03, //case battery
            0x64,
            0x00, //sound leak
            0x00, 
            0x00, //game mode
            0x01, //prompt touch tone
            0x00, 
            0x00, //ldac 
            0x01, //dual connection
            0x00,
            0x00, // limit high volume
            0x5A, // db limit 75 - 100
            0x00, // decibel reader refresh rate: 00 real time, 01 ten seconds, 02 one min
            0x01, // auto play/pause
            0x01, // prompt wearing
            0x01, // auto power enable
            0x02, // auto power duration
            0x01, // controls touch lock
            0x01, // prompt low battery 
            0xFF, 
            0xE6, //checksum 
        ];
        let (_, packet) = packet::Inbound::take::<VerboseError<_>>(input).unwrap();
        //dbg!(A3947StateUpdatePacket::take::<VerboseError<_>>(&packet.body));
        A3947StateUpdatePacket::take::<VerboseError<_>>(&packet.body)
            .expect("it should parse successfully as a A3947 state update packet");
    }
}
