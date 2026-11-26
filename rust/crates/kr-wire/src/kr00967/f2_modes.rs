//! `kr_static`, KR00967: see `docs/evidence/content/f2-f5-f3-ida-anchors.json`
//! for artifact identity and locators. RTTI names alone do not give wire IDs
//! for PTimebombClient, PTimebombServer, WeeklyLadderPacket or ConfusedFightPacket.
use super::{put, uint};
use crate::Width;
use std::io;

/// Create-dialog switch at `0xC6C080`. Distinct IDs can select the same label.
/// This table does not establish a create-room or Start121 field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GameTypeId {
    Default = 0,
    Rehab = 10,
    Hedgehog = 11,
    FlagHunt = 12,
    TimeBomb = 13,
    NpcBattle = 14,
    AiBattle = 16,
    NpcBattle17 = 17,
    SuperFighter = 18,
    Popcade = 19,
    SnowBallFight = 20,
    SuperFighter21 = 21,
    SuperFighter25 = 25,
    IamBoss = 27,
    HiddenCatchNormalMode = 28,
    HiddenCatchCompetitionMode = 29,
    IamWriggle = 43,
    All = 100,
}

impl From<GameTypeId> for u32 {
    fn from(id: GameTypeId) -> Self {
        id as Self
    }
}

impl TryFrom<u32> for GameTypeId {
    type Error = io::Error;

    fn try_from(id: u32) -> io::Result<Self> {
        use GameTypeId::*;
        Ok(match id {
            0 => Default,
            10 => Rehab,
            11 => Hedgehog,
            12 => FlagHunt,
            13 => TimeBomb,
            14 => NpcBattle,
            16 => AiBattle,
            17 => NpcBattle17,
            18 => SuperFighter,
            19 => Popcade,
            20 => SnowBallFight,
            21 => SuperFighter21,
            25 => SuperFighter25,
            27 => IamBoss,
            28 => HiddenCatchNormalMode,
            29 => HiddenCatchCompetitionMode,
            43 => IamWriggle,
            100 => All,
            _ => return Err(io::Error::other("unsupported F2 game type")),
        })
    }
}

/// `0xE67470` reads one typed `u8` selector before the player lookup. Slot
/// membership and bounds depend on the caller's context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RebirthToGhost {
    pub selector: u8,
}

impl RebirthToGhost {
    /// Static dispatch in the unpacked image (SHA-256
    /// `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`):
    /// `0xE5D3A0` indexes the table at `0xE5DE9C` by opcode minus 20, then
    /// jumps through `0xE5DDC4`. 185 goes to `0xE5D765` → `0xE67470`;
    /// 165 goes to `0xE5D570` → `0xE63260` (pickup). Rebirth has no live capture.
    pub const OPCODE: u32 = 185;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        Ok(Self {
            selector: uint(body, Width::U8)? as u8,
        })
    }

    pub fn encode(self) -> io::Result<Vec<u8>> {
        put(u64::from(self.selector), Width::U8)
    }
}

/// Ordered 16-bit entries from the config key `MapGroupVisible`. Writer
/// `0xE04CB0` copies two bytes per entry and passes the explicit byte count
/// at `0xE04DA9`. Reader `0xE04DD0` takes a buffer of `0xBB8` bytes. This
/// codec carries raw little-endian config bytes without a terminator or framing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapGroupVisible(pub Vec<u16>);

impl MapGroupVisible {
    pub const CONFIG_KEY: &str = "MapGroupVisible";
    const MAX_BYTES: usize = 0xbb8;

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > Self::MAX_BYTES {
            return Err(io::Error::other("oversized MapGroupVisible value"));
        }
        let (entries, trailing) = bytes.as_chunks::<2>();
        if !trailing.is_empty() {
            return Err(io::Error::other("partial MapGroupVisible entry"));
        }
        Ok(Self(
            entries.iter().copied().map(u16::from_le_bytes).collect(),
        ))
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        if self.0.len() > Self::MAX_BYTES / 2 {
            return Err(io::Error::other("oversized MapGroupVisible value"));
        }
        Ok(self
            .0
            .iter()
            .flat_map(|entry| entry.to_le_bytes())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_type_accepts_exact_switch_values_without_aliases() {
        let known = [
            0, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 21, 25, 27, 28, 29, 43, 100,
        ];
        for id in (0..=255).chain([256, 266, u32::MAX]) {
            match GameTypeId::try_from(id) {
                Ok(game_type) => {
                    assert!(known.contains(&id));
                    assert_eq!(u32::from(game_type), id);
                }
                Err(_) => assert!(!known.contains(&id)),
            }
        }
        assert_eq!(GameTypeId::try_from(10).unwrap(), GameTypeId::Rehab);
        assert_eq!(GameTypeId::try_from(13).unwrap(), GameTypeId::TimeBomb);
    }

    #[test]
    fn rebirth_is_exactly_one_typed_selector() {
        assert_eq!(RebirthToGhost::OPCODE, 185);
        assert_eq!(RebirthToGhost { selector: 3 }.encode().unwrap(), [0x59]);
        for selector in 0..=u8::MAX {
            let packet = RebirthToGhost { selector };
            assert_eq!(
                RebirthToGhost::decode(&packet.encode().unwrap()).unwrap(),
                packet
            );
        }
        assert!(RebirthToGhost::decode(&[]).is_err());
        assert!(RebirthToGhost::decode(&[0x59, 0x5a]).is_err());
        let pickup = crate::kr00967::pickup165(0, 3, 1, 2, 0, 2).unwrap();
        assert!(RebirthToGhost::decode(&pickup).is_err());
    }

    #[test]
    fn map_group_visibility_preserves_little_endian_order_and_rejects_partial_words() {
        let bytes = [0x34, 0x12, 0, 0, 0xff, 0xff, 0x34, 0x12];
        let visibility = MapGroupVisible(vec![0x1234, 0, u16::MAX, 0x1234]);
        assert_eq!(MapGroupVisible::decode(&bytes).unwrap(), visibility);
        assert_eq!(visibility.encode().unwrap(), bytes);
        assert_eq!(MapGroupVisible::decode(&[]).unwrap().encode().unwrap(), []);
        for n in (1..bytes.len()).step_by(2) {
            assert!(MapGroupVisible::decode(&bytes[..n]).is_err());
        }
        let limit = MapGroupVisible(vec![0x0123; 1500]);
        assert_eq!(
            MapGroupVisible::decode(&limit.encode().unwrap()).unwrap(),
            limit
        );
        assert!(MapGroupVisible::decode(&[0; 3002]).is_err());
        assert!(MapGroupVisible(vec![0; 1501]).encode().is_err());
    }
}
