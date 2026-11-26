//! Pack-exported map identity and bounded packet consistency before bind.
//! This validates input agreement; a profile is not native gameplay acceptance.
use crate::{
    ActorProfile, FixturePin, Reply,
    room::PlayerId,
    terrain::{BIG_BATTLE_SIZE, STANDARD_SIZE},
};
use kr_wire::{Width, kr00967 as wire, read_uint};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    io,
};

#[derive(Deserialize)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct Profile {
    name: String,
    map_code: u16,
    width: u8,
    height: u8,
    spawns: Vec<[u8; 2]>,
    scripted: bool,
    payload_sha256: String,
    #[serde(default)]
    adapter: Option<String>,
    #[serde(default)]
    disable_bombs: bool,
    #[serde(default)]
    drops: Vec<(String, u32)>,
    #[serde(default)]
    pack_items: Option<BTreeMap<String, u32>>,
}
fn error() -> io::Error {
    io::Error::other("inconsistent or unsupported pack map profile")
}
fn number(bytes: &[u8], width: Width) -> io::Result<u64> {
    read_uint(bytes, width).map_err(io::Error::other)
}
fn stream(body: &[u8]) -> io::Result<(Vec<u8>, usize)> {
    let size = number(body.get(..4).ok_or_else(error)?, Width::U32)? as usize;
    if !(6..=4096).contains(&size) {
        return Err(error());
    }
    let end = 4 + size;
    let decoded = wire::decode_bytes(body.get(4..end).ok_or_else(error)?)?;
    if decoded[0] != 24
        || u32::from_le_bytes(decoded[1..5].try_into().expect("bounded")) as usize != size - 5
    {
        return Err(error());
    }
    Ok((decoded, end))
}
fn adler_zero(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (0_u32, 0_u32);
    for byte in bytes {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
impl Profile {
    fn grid_size(&self) -> (u8, u8) {
        (self.width, self.height)
    }

    fn is_f1e(&self) -> bool {
        self.adapter.as_deref() == Some("f1e-adapter-r1")
    }

    fn supported(&self) -> bool {
        match self.adapter.as_deref() {
            None => self.grid_size() == STANDARD_SIZE && !self.scripted && !self.disable_bombs,
            Some("f1e-adapter-r1") => self.supported_f1e(),
            Some("f2f3-probe-r1") => self.supported_f2f3_probe(),
            Some(_) => false,
        }
    }

    fn supported_f1e(&self) -> bool {
        // Exact code/name pairs from the KR 00967 script inventory. The marker
        // alone cannot admit a different map, grid or spawn count.
        let scripted8 = ((STANDARD_SIZE, true, false), 8);
        let scripted4 = ((STANDARD_SIZE, true, false), 4);
        let (expected, expected_spawns) = match (self.map_code, self.name.as_str()) {
            (1, "Aqua01")
            | (2, "Aqua02")
            | (3, "Aqua03")
            | (4, "Aqua04")
            | (5, "Aqua05")
            | (6, "Aqua06")
            | (7, "Aqua07")
            | (8, "Aqua08")
            | (9, "Aqua09")
            | (862, "NXCL_Aqua01")
            | (167, "CaJJ_Space02")
            | (168, "CaJJ_Space08")
            | (321, "Fm_Space01")
            | (322, "Fm_Space02")
            | (1113, "WL_Space08")
            | (871, "NXCL_PangLand15")
            | (885, "PangLand00")
            | (886, "PangLand02")
            | (887, "PangLand03")
            | (888, "PangLand04")
            | (889, "PangLand06")
            | (890, "PangLand07")
            | (891, "PangLand11")
            | (892, "PangLand12")
            | (893, "Pangland14")
            | (894, "PangLand15")
            | (895, "PangLand16") => scripted8,
            (301, "Fm_Coloring03") | (325, "Fm_Wriggle02") | (326, "Fm_Wriggle04") => scripted4,
            (code @ 171..=177, name) if name == format!("CaJJ_Wriggle{:02}", code - 170) => {
                scripted4
            }
            (code @ 318..=320, name) if name == format!("Fm_RescueET{:02}", code - 317) => {
                scripted4
            }
            (code @ 1063..=1070, name) if name == format!("Space{:02}", code - 1062) => scripted8,
            (81, "BSBigCamp08") | (82, "BSBigPatrit14") | (83, "BSBigVillage10") => {
                ((BIG_BATTLE_SIZE, false, false), 8)
            }
            (code @ 467..=476, name) if name == format!("LovelyRoom{:02}", code - 466) => {
                ((STANDARD_SIZE, true, true), 8)
            }
            (code @ 929..=934, name) if name == format!("RetroCampN{:02}", code - 928) => scripted8,
            (code @ 935..=940, name) if name == format!("RetroFactoryN{:02}", code - 934) => {
                scripted8
            }
            (code @ 941..=946, name) if name == format!("RetroForestN{:02}", code - 940) => {
                scripted8
            }
            (code @ 947..=952, name) if name == format!("RetroPatritN{:02}", code - 946) => {
                scripted8
            }
            (code @ 953..=958, name) if name == format!("RetroVillageN{:02}", code - 952) => {
                scripted8
            }
            (code @ 959..=963, name) if name == format!("RetroWriggleN{:02}", code - 958) => {
                scripted4
            }
            _ => return false,
        };
        (self.grid_size(), self.scripted, self.disable_bombs) == expected
            && self.spawns.len() == expected_spawns
            && self
                .spawns
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                == expected_spawns
    }

    fn supported_f2f3_probe(&self) -> bool {
        let expected_spawns = match (self.map_code, self.name.as_str()) {
            (623, "MonSeal01") | (1051, "SnowyFields07") | (926, "RescueET01") => 4,
            (1143, "Zombie04") | (94, "CAhero01") | (1045, "SnowyFields01") | (204, "Clover01") => {
                8
            }
            _ => return false,
        };
        self.grid_size() == STANDARD_SIZE
            && self.scripted
            && !self.disable_bombs
            && self.spawns.len() == expected_spawns
            && self
                .spawns
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                == expected_spawns
    }

    pub(crate) fn load(pin: &FixturePin) -> io::Result<Self> {
        let mut value: Self = serde_json::from_slice(&pin.load()?).map_err(io::Error::other)?;
        if let Some(items) = &value.pack_items {
            crate::item_scope::apply_pack(&value.name, &mut value.drops, items)?;
        } else {
            crate::item_scope::apply(&value.name, &mut value.drops)?;
        }
        if !value.supported()
            || !(2..=8).contains(&value.spawns.len())
            || value
                .spawns
                .iter()
                .any(|[x, y]| *x >= value.width || *y >= value.height)
            || !(1..=64).contains(&value.name.len())
            || !value
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || value.payload_sha256.len() != 64
            || !value.payload_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(error());
        }
        Ok(value)
    }
    pub(crate) fn validate(
        &self,
        expected_count: usize,
        packets: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
    ) -> io::Result<()> {
        if self.is_f1e() {
            // Standalone profiles have no choice-owned rule binding.
            return Err(error());
        }
        self.validate_selected(expected_count, packets, actors, 0, 1)
    }
    pub(crate) fn validate_selected(
        &self,
        expected_count: usize,
        packets: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
        selected: usize,
        count: usize,
    ) -> io::Result<()> {
        let start = &packets["start"].payload;
        let wire::Start121Map {
            map_code: code,
            name,
            dynamic_index,
            ..
        } = wire::start121_map_context(start)?;
        let selection = packets["map"].payload.get(..8).ok_or_else(error)?;
        if code != self.map_code
            || name != self.name.as_bytes()
            || wire::map117_selection(selection)? != (code, selected as u32)
            || number(&packets["play"].payload[5..7], Width::U16)? != u64::from(code)
        {
            return Err(error());
        }
        // Native selects coordinates by this exported spawn ordinal; do not
        // replace pack coordinates with fixture coordinates from another map.
        let (start_stream, _) = stream(&start[1..])?;
        let rows = 43 + name.len();
        if usize::from(start_stream[rows - 5]) != expected_count || dynamic_index != selected as u32
        {
            return Err(error());
        }
        if start_stream[rows..]
            .as_chunks::<26>()
            .0
            .iter()
            .any(|row| usize::from(row[2]) >= self.spawns.len())
        {
            return Err(error());
        }
        let (table, end) = stream(&packets["table"].payload)?;
        if end != packets["table"].payload.len() {
            return Err(error());
        }
        if table[5] == 0 {
            let inner = &table[6..];
            let rows = table_rows(inner)?;
            if rows.len() != count
                || rows.get(selected).is_none_or(|row| {
                    row.code != code
                        || row.name != self.name.as_bytes()
                        || row.id != selected as u32
                })
            {
                return Err(error());
            }
        } else if crate::sha256_hex(&table)
            != "9b30734b4feccf44fba7432f65137d7125f13e2580aea761749cca62543d98eb"
            || (code, self.name.as_str()) != (278, "ExVillage01")
            || selected != 0
            || count != 1
        {
            // Preserve the independently measured compressed baseline exactly.
            // New map tables use the already measured raw-selector0 schema;
            // no generic compressed decoder or inferred checksum is introduced.
            return Err(error());
        }
        let checksum = adler_zero(&table[5..]);
        for actor in actors.values() {
            let ready = &actor.packets["ready"].payload;
            let (_, end) = stream(ready)?;
            if ready.len() != end + 13
                || number(&ready[end..end + 4], Width::U32)? != u64::from(checksum)
            {
                return Err(error());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(map_code: u16, name: &str, spawn_count: usize) -> Profile {
        Profile {
            name: name.to_owned(),
            map_code,
            width: STANDARD_SIZE.0,
            height: STANDARD_SIZE.1,
            spawns: (0..spawn_count).map(|x| [x as u8, 0]).collect(),
            scripted: true,
            payload_sha256: "0".repeat(64),
            adapter: Some("f2f3-probe-r1".to_owned()),
            disable_bombs: false,
            drops: Vec::new(),
            pack_items: None,
        }
    }

    #[test]
    fn f2f3_probe_allowlist_accepts_exact_profiles() {
        for (map_code, name, spawn_count) in [
            (623, "MonSeal01", 4),
            (1051, "SnowyFields07", 4),
            (1143, "Zombie04", 8),
            (94, "CAhero01", 8),
            (926, "RescueET01", 4),
            (1045, "SnowyFields01", 8),
            (204, "Clover01", 8),
        ] {
            assert!(
                profile(map_code, name, spawn_count).supported(),
                "{map_code} {name}"
            );
        }
    }

    #[test]
    fn f2f3_probe_allowlist_rejects_unpinned_profiles() {
        let base = profile(623, "MonSeal01", 4);

        let mut unknown_adapter = base.clone();
        unknown_adapter.adapter = Some("unknown-adapter".to_owned());
        assert!(!unknown_adapter.supported());

        let mut missing_adapter = base.clone();
        missing_adapter.adapter = None;
        assert!(!missing_adapter.supported());

        let mut wrong_code = base.clone();
        wrong_code.map_code += 1;
        assert!(!wrong_code.supported());

        let mut wrong_spawn_count = base.clone();
        wrong_spawn_count.spawns.pop();
        assert!(!wrong_spawn_count.supported());

        let mut non_standard_grid = base.clone();
        non_standard_grid.width = BIG_BATTLE_SIZE.0;
        non_standard_grid.height = BIG_BATTLE_SIZE.1;
        assert!(!non_standard_grid.supported());

        let mut unlisted_scripted = base.clone();
        unlisted_scripted.map_code = 999;
        unlisted_scripted.name = "UnlistedScript".to_owned();
        assert!(!unlisted_scripted.supported());

        let mut not_scripted = base.clone();
        not_scripted.scripted = false;
        assert!(!not_scripted.supported());

        let mut bombs_disabled = base;
        bombs_disabled.disable_bombs = true;
        assert!(!bombs_disabled.supported());
    }

    #[test]
    fn default_and_f1e_adapter_paths_remain_unchanged() {
        let mut default_profile = profile(623, "MonSeal01", 4);
        default_profile.adapter = None;
        default_profile.scripted = false;
        assert!(default_profile.supported());

        let mut f1e_profile = profile(3, "Aqua03", 8);
        f1e_profile.adapter = Some("f1e-adapter-r1".to_owned());
        assert!(f1e_profile.supported());

        let mut f2f3_under_f1e = profile(623, "MonSeal01", 4);
        f2f3_under_f1e.adapter = Some("f1e-adapter-r1".to_owned());
        assert!(!f2f3_under_f1e.supported());
    }
}

struct TableRow<'a> {
    id: u32,
    code: u16,
    name: &'a [u8],
}
fn table_rows(inner: &[u8]) -> io::Result<Vec<TableRow<'_>>> {
    let count = u16::from_le_bytes(
        inner
            .get(..2)
            .ok_or_else(error)?
            .try_into()
            .expect("bounded"),
    ) as usize;
    if !(1..=16).contains(&count) {
        return Err(error());
    }
    let mut at = 2;
    let mut rows = Vec::new();
    for _ in 0..count {
        let id = u32::from_le_bytes(
            inner
                .get(at..at + 4)
                .ok_or_else(error)?
                .try_into()
                .expect("bounded"),
        );
        at += 5;
        let mut names = Vec::new();
        for _ in 0..2 {
            for _ in 0..2 {
                let n = u32::from_le_bytes(
                    inner
                        .get(at..at + 4)
                        .ok_or_else(error)?
                        .try_into()
                        .expect("bounded"),
                ) as usize;
                if n > 64 {
                    return Err(error());
                }
                let value = inner.get(at + 4..at + 4 + n).ok_or_else(error)?;
                names.push(value);
                at += 4 + n;
            }
            at += 2;
        }
        at += 7;
        let code = u16::from_le_bytes(
            inner
                .get(at..at + 2)
                .ok_or_else(error)?
                .try_into()
                .expect("bounded"),
        );
        at += 7;
        rows.push(TableRow {
            id,
            code,
            name: names[1],
        });
    }
    if at != inner.len() {
        return Err(error());
    }
    Ok(rows)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChoiceConfig {
    pub map_profile: FixturePin,
    pub packets: HashMap<String, crate::LobbyPacketPin>,
    #[serde(default)]
    pub terrain: Option<crate::terrain::Config>,
    #[serde(default)]
    pub bomb_cancellation: Option<crate::rules::BombCancellationConfig>,
}
pub(crate) struct Choice {
    pub code: u16,
    pub index: u32,
    pub spawn_count: usize,
    pub bodies: HashMap<crate::room_packets::Template, Vec<u8>>,
    pub floor: crate::pickups::Catalog,
    pub terrain: Option<crate::terrain::Terrain>,
    pub bomb_cancellation: Option<crate::rules::BombCancellationCells>,
}
pub(crate) struct Catalog {
    pub choices: Vec<Choice>,
}
impl Catalog {
    pub(crate) fn load(
        configs: &[ChoiceConfig],
        count: usize,
        root: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
        needle: bool,
        item_policy: crate::pickups::Policy,
        managed_template: bool,
    ) -> io::Result<Self> {
        use crate::room_packets::Template;
        if !(2..=16).contains(&configs.len()) {
            return Err(error());
        }
        let mut choices = Vec::new();
        let mut codes = std::collections::HashSet::new();
        for (index, cfg) in configs.iter().enumerate() {
            if cfg.terrain.is_some() && !item_policy.enabled() {
                return Err(io::Error::other("map terrain requires basic-item policy"));
            }
            if cfg.packets.len() != 4 {
                return Err(error());
            }
            let profile = Profile::load(&cfg.map_profile)?;
            let mut packets = root.clone();
            let mut bodies = HashMap::new();
            for (name, key) in [
                ("map", Template::Map117),
                ("start", Template::Start121),
                ("play", Template::Play196),
                ("floor", Template::Floor582),
            ] {
                let pin = cfg.packets.get(name).ok_or_else(error)?;
                let raw = pin.fixture.load()?;
                let frame = kr_wire::decode(&raw, pin.sequence.ok_or_else(error)?)
                    .map_err(io::Error::other)?;
                if pin.opcode != key.opcode()
                    || frame.opcode != key.opcode()
                    || !wire::header_matches(&raw[..37])
                {
                    return Err(error());
                }
                bodies.insert(key, frame.payload.clone());
                packets.insert(
                    name.into(),
                    Reply {
                        header: raw[..37].try_into().expect("decoded"),
                        opcode: frame.opcode,
                        payload: frame.payload,
                    },
                );
            }
            let template_count = if managed_template {
                count.min(profile.spawns.len())
            } else {
                count
            };
            profile.validate_selected(template_count, &packets, actors, index, configs.len())?;
            if !codes.insert(profile.map_code) {
                return Err(error());
            }
            let floor = crate::pickups::Catalog::from_floor(&packets["floor"].payload)?;
            floor.validate_needle(needle)?;
            floor.validate_stat_policy(item_policy)?;
            let terrain = cfg
                .terrain
                .as_ref()
                .map(|terrain| {
                    terrain.validate_for_map(
                        &cfg.packets["map"].fixture.load()?,
                        &cfg.packets["start"].fixture.load()?,
                        &profile.payload_sha256,
                    )
                })
                .transpose()?;
            if terrain
                .as_ref()
                .is_some_and(|t| t.grid_size() != profile.grid_size())
                || (profile.is_f1e()
                    && terrain
                        .as_ref()
                        .is_none_or(|t| t.bombs_disabled() != profile.disable_bombs))
            {
                return Err(error());
            }
            choices.push(Choice {
                code: profile.map_code,
                index: index as u32,
                spawn_count: profile.spawns.len(),
                bodies,
                floor,
                terrain,
                bomb_cancellation: cfg
                    .bomb_cancellation
                    .as_ref()
                    .map(|rule| match profile.grid_size() {
                        STANDARD_SIZE => rule.validate(&profile.payload_sha256),
                        grid => rule.validate_for_grid(&profile.payload_sha256, grid),
                    })
                    .transpose()?,
            });
        }
        for (name, key) in [
            ("map", Template::Map117),
            ("start", Template::Start121),
            ("play", Template::Play196),
            ("floor", Template::Floor582),
        ] {
            if root[name].payload != choices[0].bodies[&key] {
                return Err(error());
            }
        }
        Ok(Self { choices })
    }
    pub(crate) fn select(&self, code: u16, index: u32) -> io::Result<usize> {
        self.choices
            .iter()
            .position(|c| c.code == code && c.index == index)
            .ok_or_else(error)
    }
}
