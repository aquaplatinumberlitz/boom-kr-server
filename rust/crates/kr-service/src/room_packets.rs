//! Legacy JSON aliases end at this configuration adapter.
//! Runtime uses typed body templates and the single measured header.
use crate::Reply;
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Template {
    Create86,
    Join87,
    Listing91,
    Table238,
    Map117,
    Floor582,
    Start121,
    Go197,
    Play196,
    Peer153,
    SlotOneWins175,
    SlotZeroWins175,
    AllDead175,
    Return180,
}
impl Template {
    pub(crate) fn opcode(self) -> u32 {
        match self {
            Self::Create86 => 86,
            Self::Join87 => 87,
            Self::Listing91 => 91,
            Self::Table238 => 238,
            Self::Map117 => 117,
            Self::Floor582 => 582,
            Self::Start121 => 121,
            Self::Go197 => 197,
            Self::Play196 => 196,
            Self::Peer153 => 153,
            Self::SlotOneWins175 => 175,
            Self::SlotZeroWins175 => 175,
            Self::AllDead175 => 175,
            Self::Return180 => 180,
        }
    }
}
pub(crate) fn normalize_legacy(packets: &HashMap<String, Reply>) -> HashMap<Template, Vec<u8>> {
    [
        ("create", Template::Create86),
        ("join", Template::Join87),
        ("listing", Template::Listing91),
        ("table", Template::Table238),
        ("map", Template::Map117),
        ("floor", Template::Floor582),
        ("start", Template::Start121),
        ("go", Template::Go197),
        ("play", Template::Play196),
        ("peer", Template::Peer153),
        ("rank", Template::SlotOneWins175),
        ("rank_Awin", Template::SlotZeroWins175),
        ("rank_all_dead", Template::AllDead175),
        ("return", Template::Return180),
    ]
    .into_iter()
    .filter_map(|(alias, key)| {
        packets
            .get(alias)
            .map(|packet| (key, packet.payload.clone()))
    })
    .collect()
}
pub(crate) fn packet(opcode: u32, payload: Vec<u8>) -> Reply {
    Reply {
        header: kr_wire::kr00967::HEADER,
        opcode,
        payload,
    }
}
