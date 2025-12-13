use std::{ffi::OsStr, fs::OpenOptions, io::Read, path::Path};

use nom::{
    bytes::complete::take,
    combinator::{map, verify},
    multi::{count, many_till, many0},
    number::complete::{le_f32, le_i8, le_i16, le_i32, le_u8, le_u16, le_u32},
    sequence::tuple,
};

use crate::{
    nom_helper::{Result, nom_fail, take_point_float},
    parse_netmsg,
    types::{
        Aux, AuxRefCell, ClientData, ConsoleCommand, Demo, DemoBuffer, DemoInfo, Directory,
        DirectoryEntry, Event, EventArgs, Frame, FrameData, Header, MessageData,
        MessageDataParseMode, MoveVars, NetworkMessage, NetworkMessageType, RefParams,
        SequenceInfo, Sound, UserCmd, WeaponAnimation,
    },
};

pub enum MsgDataParseMode {
    Parse,
    Raw,
    None,
}

impl Demo {
    pub fn parse_from_file(
        path: impl AsRef<OsStr> + AsRef<Path>,
        mode: MsgDataParseMode,
    ) -> eyre::Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut bytes: Vec<u8> = vec![];

        file.read_to_end(&mut bytes)?;
    }
}
pub fn parse_demo(i: &[u8], parse_mode: MsgDataParseMode) -> Result<Demo> {}
