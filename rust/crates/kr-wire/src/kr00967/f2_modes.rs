//! KR00967 `kr_static` provenance from PR #810. See
//! `docs/protocol/f2-bula-tail-and-mode-wire-re.md` and
//! `docs/evidence/content/f2-bula-tail-and-mode-wire-re.json` for locators and
//! pack hashes. IDA DB SHA-256: `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`.
//! CA.exe SHA-256: `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`.
use super::{put, uint};
use crate::Width;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F2AnchorStatus {
    /// Confirmed for the stated static finding only.
    KrStaticConfirmed,
    UnmeasuredPendingWineCapture,
}

/// Function addresses are IDA VAs in the KR00967 image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F2AnchorSite {
    Function(u32),
    ServerToClientHandler { va: u32, opcode: u32 },
    Rtti,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F2ModeAnchor {
    pub name: &'static str,
    pub site: F2AnchorSite,
    pub status: F2AnchorStatus,
    pub finding: &'static str,
}

impl F2ModeAnchor {
    const fn confirmed(name: &'static str, site: F2AnchorSite, finding: &'static str) -> Self {
        Self {
            name,
            site,
            status: F2AnchorStatus::KrStaticConfirmed,
            finding,
        }
    }

    const fn rtti(name: &'static str) -> Self {
        Self::confirmed(name, F2AnchorSite::Rtti, "Name only; live wire ID unknown.")
    }
}

/// Static locators carry no family binding or server authority.
pub static F2_MODE_ANCHORS: &[F2ModeAnchor] = &[
    F2ModeAnchor::confirmed(
        "sub_57F520",
        F2AnchorSite::Function(0x57F520),
        "The roomtype string is used in room creation; the numeric wire field is unknown.",
    ),
    F2ModeAnchor::confirmed(
        "sub_C6C080",
        F2AnchorSite::Function(0xC6C080),
        "The GameType switch selects UI labels; those IDs do not establish wire values.",
    ),
    F2ModeAnchor::confirmed(
        "MapGroupVisible writer",
        F2AnchorSite::Function(0xE04CB0),
        "Writes 16-bit entries to the MapGroupVisible config key.",
    ),
    F2ModeAnchor::confirmed(
        "MapGroupVisible reader",
        F2AnchorSite::Function(0xE04DD0),
        "Reads the config entries used in map and group selection.",
    ),
    F2ModeAnchor::confirmed(
        "CGamePlayMap Soul reader",
        F2AnchorSite::Function(0x115C770),
        "The loader calls virtual slot +0x64 after drop and placement; name lookup is at 0x11676E0.",
    ),
    F2ModeAnchor::confirmed(
        "CGamePlayMap Soul record consumer",
        F2AnchorSite::Function(0x115DCF0),
        "Consumes records after the name lookup; per-map timing and authority are unknown.",
    ),
    F2ModeAnchor::confirmed(
        "RebirthToGhost",
        F2AnchorSite::ServerToClientHandler {
            va: 0xE67470,
            opcode: RebirthToGhost::OPCODE,
        },
        "The stage jump table binds S->C 185 to a u8 slot reader and entity vtable call; no live capture.",
    ),
    F2ModeAnchor::rtti("WeeklyLadderPacket"),
    F2ModeAnchor::rtti("ConfusedFightPacket"),
    F2ModeAnchor::rtti("PTimebombClient"),
    F2ModeAnchor::rtti("PTimebombServer"),
    F2ModeAnchor::rtti("PWeeklyLadderUserInfoResult"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F2ModeFamily {
    pub family: &'static str,
    pub wire_mode: F2AnchorStatus,
    pub round_objective_wire: F2AnchorStatus,
    /// Candidate UI associations only.
    pub static_game_type_candidates: &'static [GameTypeId],
}

impl F2ModeFamily {
    const fn pending(
        family: &'static str,
        static_game_type_candidates: &'static [GameTypeId],
    ) -> Self {
        Self {
            family,
            wire_mode: F2AnchorStatus::UnmeasuredPendingWineCapture,
            round_objective_wire: F2AnchorStatus::UnmeasuredPendingWineCapture,
            static_game_type_candidates,
        }
    }
}

/// The 18 families in the #810 evidence matrix, with no measured mode or objective wire.
pub static F2_MODE_FAMILIES: &[F2ModeFamily] = &[
    F2ModeFamily::pending("Baseball", &[]),
    F2ModeFamily::pending("Bula", &[]),
    F2ModeFamily::pending("CAPong", &[]),
    F2ModeFamily::pending("Clover", &[]),
    F2ModeFamily::pending("Coloring", &[]),
    F2ModeFamily::pending("Gumiho", &[]),
    F2ModeFamily::pending("Hamster", &[]),
    F2ModeFamily::pending("Kart", &[]),
    F2ModeFamily::pending("PopcadeA", &[GameTypeId::Popcade]),
    F2ModeFamily::pending("PopcadeB", &[GameTypeId::Popcade]),
    F2ModeFamily::pending("PopcadeC", &[GameTypeId::Popcade]),
    F2ModeFamily::pending("Rabbit", &[]),
    F2ModeFamily::pending("RescueET", &[]),
    F2ModeFamily::pending("SnowyFields", &[]),
    F2ModeFamily::pending(
        "SuperFighter",
        &[
            GameTypeId::SuperFighter,
            GameTypeId::SuperFighter21,
            GameTypeId::SuperFighter25,
        ],
    ),
    F2ModeFamily::pending("Wriggle", &[]),
    F2ModeFamily::pending("Zombie", &[]),
    F2ModeFamily::pending("Zzolman", &[]),
];

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

/// Ordered raw `(x, y)` pairs from one group in a Bula map tail.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BulaGroupPoints(pub Vec<(i32, i32)>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BulaTailRecord {
    /// Length-prefixed name bytes, including any NUL padding in the pack.
    pub raw_cp949_name: Vec<u8>,
    pub x: u8,
    pub y: u8,
    pub fields: [i32; 3],
}

/// PR #810 `kr_static/confirmed` little-endian `group_record_tail` at the
/// end of a map payload. Point coordinates, record fields, and script content
/// are stored as supplied; the codec does not assign gameplay semantics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BulaGroupRecordTail {
    pub groups: Vec<BulaGroupPoints>,
    pub records: Vec<BulaTailRecord>,
    pub script: Vec<u8>,
}

impl BulaGroupRecordTail {
    // Allocation limits from tools/kr_f1e_script_audit.py, not client limits.
    pub const MAX_GROUPS: usize = 256;
    pub const MAX_POINTS_PER_GROUP: usize = 4096;
    pub const MAX_RECORDS: usize = 256;
    pub const MAX_NAME_BYTES: usize = 64;
    /// Codec allocation limit; all seven Bula assets in #810 have empty scripts.
    pub const MAX_SCRIPT_BYTES: usize = 1024 * 1024;

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut reader = BulaTailReader(bytes);
        let group_count = reader.count(Self::MAX_GROUPS, "group count")?;
        reader.require_entries(group_count, 4)?;
        let mut groups = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            let point_count = reader.count(Self::MAX_POINTS_PER_GROUP, "point count")?;
            reader.require_entries(point_count, 8)?;
            let mut points = Vec::with_capacity(point_count);
            for _ in 0..point_count {
                points.push((reader.i32()?, reader.i32()?));
            }
            groups.push(BulaGroupPoints(points));
        }

        let record_count = reader.count(Self::MAX_RECORDS, "record count")?;
        reader.require_entries(record_count, 4 + 1 + 14)?;
        let mut records = Vec::with_capacity(record_count);
        for _ in 0..record_count {
            let name_len = reader.count(Self::MAX_NAME_BYTES, "name length")?;
            if name_len == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "empty Bula name",
                ));
            }
            let name = reader.take(name_len)?;
            let [x, y] = reader.array()?;
            let fields = [reader.i32()?, reader.i32()?, reader.i32()?];
            records.push(BulaTailRecord {
                raw_cp949_name: name.to_vec(),
                x,
                y,
                fields,
            });
        }

        let script_len = u32::from_le_bytes(reader.array()?) as usize;
        if script_len > Self::MAX_SCRIPT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "oversized Bula script",
            ));
        }
        let script = reader.take(script_len)?;
        if !reader.0.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing bytes after Bula script",
            ));
        }

        Ok(Self {
            groups,
            records,
            script: script.to_vec(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        if self.script.len() > Self::MAX_SCRIPT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "oversized Bula script",
            ));
        }

        let mut bytes = Vec::new();
        write_bula_count(
            &mut bytes,
            self.groups.len(),
            Self::MAX_GROUPS,
            "group count",
        )?;
        for group in &self.groups {
            write_bula_count(
                &mut bytes,
                group.0.len(),
                Self::MAX_POINTS_PER_GROUP,
                "point count",
            )?;
            for &(x, y) in &group.0 {
                bytes.extend_from_slice(&x.to_le_bytes());
                bytes.extend_from_slice(&y.to_le_bytes());
            }
        }
        write_bula_count(
            &mut bytes,
            self.records.len(),
            Self::MAX_RECORDS,
            "record count",
        )?;
        for record in &self.records {
            if record.raw_cp949_name.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "empty Bula name",
                ));
            }
            write_bula_count(
                &mut bytes,
                record.raw_cp949_name.len(),
                Self::MAX_NAME_BYTES,
                "name length",
            )?;
            bytes.extend_from_slice(&record.raw_cp949_name);
            bytes.extend_from_slice(&[record.x, record.y]);
            for field in record.fields {
                bytes.extend_from_slice(&field.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(self.script.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.script);
        Ok(bytes)
    }
}

struct BulaTailReader<'a>(&'a [u8]);

impl<'a> BulaTailReader<'a> {
    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let (head, tail) = self
            .0
            .split_at_checked(len)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        self.0 = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let (head, tail) = self
            .0
            .split_first_chunk()
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        self.0 = tail;
        Ok(*head)
    }

    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn count(&mut self, cap: usize, field: &str) -> io::Result<usize> {
        usize::try_from(self.i32()?)
            .ok()
            .filter(|&count| count <= cap)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, format!("invalid Bula {field}"))
            })
    }

    fn require_entries(&self, count: usize, min_size: usize) -> io::Result<()> {
        if count > self.0.len() / min_size {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(())
    }
}

fn write_bula_count(bytes: &mut Vec<u8>, len: usize, cap: usize, field: &str) -> io::Result<()> {
    if len > cap {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("oversized Bula {field}"),
        ));
    }
    bytes.extend_from_slice(&(len as i32).to_le_bytes());
    Ok(())
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
