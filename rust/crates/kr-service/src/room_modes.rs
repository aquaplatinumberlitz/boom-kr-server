//! F2 operator policy, rule descriptors and static KR00967 catalog. UI IDs
//! are opt-in aliases only. Attr membership and candidate links only filter.
//! KR00967 static anchors: MapGroupVisible writer 0xE04CB0 and reader 0xE04DD0;
//! create dialogs 0xC541E0 / 0xC55AF0 (Ladder), 0xC8DF50 (Mission), and
//! roomtype 0x57F520. These do not establish a live F2 packet contract.
use kr_wire::kr00967::{GameTypeId, MapGroupVisible, RoomTeamSelector, Start121Map};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub(crate) mod adapter;
mod rules;
mod runtime;
use rules::{ExtendedRules, RuleTable, Wave};

const CATALOG: &str = include_str!("../../../../docs/evidence/content/f2-mode-catalog.json");

// Families in the KR00967 terrain export allowlist (through PR #789). The catalog
// provides exact map names and candidate groups, but no native start route.
const EXPORTED_F2_FAMILIES: &[&str] = &[
    "CAPong",
    "Clover",
    "Coloring",
    "Hamster",
    "Rabbit",
    "RescueET",
    "SnowyFields",
    "SuperFighter",
    "Wriggle",
    "Zombie",
    "Zzolman",
];

// The 21 F2? families in f2-extended-modes-matrix.json, paired with their
// literal KR00967 census entries. Pack membership classifies these maps as
// tentative; it supplies neither a room type nor a start route.
const TENTATIVE_MAP_FAMILIES: &[(&str, &[&str])] = &[
    (
        "Balansa",
        &[
            "Balansa01",
            "Balansa02",
            "Balansa03",
            "Balansa04",
            "Balansa05",
        ],
    ),
    ("Baram", &["Baram01", "Baram02", "Baram03", "Baram04"]),
    (
        "BlackLucy",
        &[
            "BlackLucy01",
            "BlackLucy02",
            "BlackLucy03",
            "BlackLucy04",
            "BlackLucy05",
            "BlackLucy06",
            "BlackLucy07",
            "BlackLucy08",
            "BlackLucy09",
        ],
    ),
    (
        "Clovertest",
        &["Clovertest_01", "Clovertest_02", "Clovertest_03"],
    ),
    (
        "ConfusedFight",
        &["ConfusedFight01", "ConfusedFight02", "ConfusedFight03"],
    ),
    (
        "DarkCastle",
        &[
            "DarkCastle01",
            "DarkCastle02",
            "DarkCastle03",
            "DarkCastle04",
            "DarkCastle05",
            "DarkCastle06",
            "DarkCastle07",
            "DarkCastle08",
            "DarkCastle09",
        ],
    ),
    (
        "DarkVampire",
        &[
            "DarkVampire01",
            "DarkVampire02",
            "DarkVampire03",
            "DarkVampire04",
            "DarkVampire05",
            "DarkVampire06",
            "DarkVampire07",
            "DarkVampire08",
        ],
    ),
    (
        "Fm_BigBattle",
        &[
            "Fm_BigBattle01",
            "Fm_BigBattle02",
            "Fm_BigBattle03",
            "Fm_BigBattle04",
        ],
    ),
    (
        "Fm_BigBattleH_PreparePlay",
        &["Fm_BigBattleH_PreparePlay01"],
    ),
    ("Fm_BigBattle_PreparePlay", &["Fm_BigBattle_PreparePlay01"]),
    (
        "GoldButterfly",
        &["GoldButterfly01", "GoldButterfly02", "GoldButterfly03"],
    ),
    ("Hiphopclub", &["Hiphopclub07"]),
    ("MagNest", &["MagNest01"]),
    (
        "NewXmas",
        &[
            "NewXmas01",
            "NewXmas02",
            "NewXmas03",
            "NewXmas04",
            "NewXmas05",
            "NewXmas06",
        ],
    ),
    (
        "OkongOfLegend",
        &["OkongOfLegend01", "OkongOfLegend02", "OkongOfLegend03"],
    ),
    ("Smash", &["Smash01", "Smash02", "Smash03", "Smash04"]),
    ("Space_TW", &["Space_TW09", "Space_TW10"]),
    (
        "UserContest",
        &[
            "UserContest06",
            "UserContest07",
            "UserContest08",
            "UserContest09",
            "UserContest10",
            "UserContest11",
            "UserContest12",
            "UserContest13",
            "UserContest14",
            "UserContest15",
            "UserContest16",
            "UserContest17",
            "UserContest18",
            "UserContest19",
            "UserContest20",
            "UserContest21",
            "UserContest22",
        ],
    ),
    ("WhiteAngel", &["WhiteAngel01", "WhiteAngel02"]),
    ("WinningStreak_Colosseum", &["WinningStreak_Colosseum01"]),
    (
        "Wriggletest",
        &[
            "Wriggletest_01",
            "Wriggletest_02",
            "Wriggletest_03",
            "Wriggletest_04",
        ],
    ),
];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum ModeKind {
    Airplane,
    Antarctic,
    Baseball,
    BigBattle,
    Capong,
    Clover,
    CoOp,
    Coloring,
    ConfusedFight,
    FarmMission,
    #[serde(rename = "free_1v7")]
    Free1v7,
    Hamster,
    Hedgehog,
    Ladder,
    LeaderCapture,
    Matchmaking,
    Popcade,
    RescueEt,
    Revive,
    SnowballEvent,
    SnowyChamp,
    SnowyFields,
    SuperFighter,
    Timebomb,
    Wriggle,
    Zombie,
    Zzolman,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum MapGroup {
    #[serde(rename = "LadderMapGroup")]
    Ladder,
    #[serde(rename = "ReBirthMapGroup")]
    Rebirth,
    // Operator visibility labels for exact catalog families. These labels
    // establish neither KR Attr locators nor default numeric MapGroupVisible IDs.
    Coloring,
    SuperFighter,
}

impl ModeKind {
    fn evaluation_map_group(self) -> Option<MapGroup> {
        match self {
            Self::Coloring => Some(MapGroup::Coloring),
            Self::SuperFighter => Some(MapGroup::SuperFighter),
            _ => None,
        }
    }
}

impl MapGroup {
    fn from_locator(locator: &str) -> Option<Self> {
        match locator.strip_prefix("FxContents/Attr/MapGroup/")? {
            "LadderMapGroup" => Some(Self::Ladder),
            "ReBirthMapGroup" => Some(Self::Rebirth),
            _ => None,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    visible_map_groups: VisibilityConfig,
    #[serde(default)]
    create_overrides: OverrideConfig,
    #[serde(default)]
    runtime_overrides: OverrideConfig,
    #[serde(default)]
    extended_rules: ExtendedRules,
    #[serde(default)]
    candidate_map_family: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VisibilityConfig {
    Named(BTreeSet<MapGroup>),
    Encoded(EncodedVisibility),
}

impl Default for VisibilityConfig {
    fn default() -> Self {
        Self::Named(BTreeSet::new())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodedVisibility {
    bytes: Vec<u8>,
    // The codec only establishes u16 entries. These bindings are supplied by
    // the operator for this input; the policy assumes no KR numeric group IDs.
    group_ids: BTreeMap<MapGroup, u16>,
}

impl VisibilityConfig {
    fn resolve(&self) -> io::Result<BTreeSet<MapGroup>> {
        match self {
            Self::Named(groups) => Ok(groups.clone()),
            Self::Encoded(input) => {
                let entries = MapGroupVisible::decode(&input.bytes)?;
                let mut by_id = BTreeMap::new();
                for (&group, &id) in &input.group_ids {
                    if by_id.insert(id, group).is_some() {
                        return Err(io::Error::other("ambiguous MapGroupVisible binding"));
                    }
                }
                entries
                    .0
                    .into_iter()
                    .map(|id| {
                        by_id.get(&id).copied().ok_or_else(|| {
                            io::Error::other(format!("unbound MapGroupVisible entry: {id}"))
                        })
                    })
                    .collect()
            }
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(untagged)]
enum Preset {
    Named(ModeKind),
    GameType(GameTypePreset),
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct GameTypePreset {
    #[serde(deserialize_with = "deserialize_game_type_id")]
    game_type_id: GameTypeId,
}

fn deserialize_game_type_id<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<GameTypeId, D::Error> {
    GameTypeId::try_from(u32::deserialize(d)?).map_err(serde::de::Error::custom)
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideConfig {
    free: Option<Preset>,
    manner: Option<Preset>,
}

impl OverrideConfig {
    fn is_empty(&self) -> bool {
        [self.free, self.manner].into_iter().all(|preset| {
            matches!(
                preset,
                None | Some(Preset::GameType(GameTypePreset {
                    game_type_id: GameTypeId::Default
                }))
            )
        })
    }

    fn resolve(&self, rules: &RuleTable) -> io::Result<CreateOverrides> {
        let resolve = |preset| match preset {
            None
            | Some(Preset::GameType(GameTypePreset {
                game_type_id: GameTypeId::Default,
            })) => Ok(None),
            Some(Preset::Named(kind)) => Ok(Some(kind)),
            Some(Preset::GameType(GameTypePreset { game_type_id })) => rules
                .mode_for_game_type(game_type_id)
                .map(Some)
                .ok_or_else(|| {
                    io::Error::other(format!("unsupported F2 game type alias: {game_type_id:?}"))
                }),
        };
        Ok(CreateOverrides {
            free: resolve(self.free)?,
            manner: resolve(self.manner)?,
        })
    }
}

/// Operator presets keyed by the unchanged native team selector.
#[derive(Clone, Copy)]
struct CreateOverrides {
    free: Option<ModeKind>,
    manner: Option<ModeKind>,
}

impl CreateOverrides {
    fn for_team(self, team: RoomTeamSelector) -> Option<ModeKind> {
        match team {
            RoomTeamSelector::Free => self.free,
            RoomTeamSelector::Manner => self.manner,
        }
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema: String,
    build: String,
    modes: Vec<Mode>,
    families: Vec<Family>,
}

#[derive(Deserialize)]
struct Mode {
    ruleset_group: ModeKind,
    wave: Wave,
    related_families: Vec<FamilyRef>,
    pack: Pack,
}

#[derive(Deserialize)]
struct FamilyRef {
    family: String,
}

#[derive(Deserialize)]
struct Pack {
    map_groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    locator: String,
    exact_census_matches: Option<Vec<GroupMatch>>,
}

#[derive(Deserialize)]
struct GroupMatch {
    entry: String,
}

#[derive(Deserialize)]
struct Family {
    family: String,
    grouping: FamilyGrouping,
    maps: Vec<MapName>,
}

#[derive(Deserialize)]
struct FamilyGrouping {
    ruleset_group: String,
}

#[derive(Deserialize)]
struct MapName {
    name: String,
}

pub(crate) struct Policy {
    catalog: Catalog,
    rules: RuleTable,
    visible: BTreeSet<MapGroup>,
    create: CreateOverrides,
    runtime: CreateOverrides,
}

impl Catalog {
    fn load() -> io::Result<Self> {
        let mut catalog: Self = serde_json::from_str(CATALOG).map_err(io::Error::other)?;
        if catalog.build != "KR00967" || catalog.schema != "kre.f2-mode-catalog.v1" {
            return Err(io::Error::other("unsupported F2 catalog build or schema"));
        }
        for &name in EXPORTED_F2_FAMILIES {
            let family = catalog
                .families
                .iter()
                .find(|family| family.family == name)
                .ok_or_else(|| io::Error::other(format!("missing F2 catalog family: {name}")))?;
            let group = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(
                &family.grouping.ruleset_group,
            );
            let kind = match (
                name,
                ModeKind::deserialize(group).map_err(io::Error::other)?,
            ) {
                // The catalog labels both farm families farm_mission. Give Hamster
                // its own operator evaluation preset without widening Rabbit's map gate.
                ("Hamster", ModeKind::FarmMission) => ModeKind::Hamster,
                (_, kind) => kind,
            };
            if let Some(mode) = catalog.modes.iter().find(|mode| mode.ruleset_group == kind) {
                if !mode.wave.in_f2()
                    || !mode.related_families.iter().any(|link| link.family == name)
                {
                    return Err(io::Error::other(format!(
                        "inconsistent F2 family group: {name}"
                    )));
                }
            } else {
                // A family link supplies exact map candidates only.
                catalog.modes.push(Mode {
                    ruleset_group: kind,
                    wave: Wave::F2Candidate,
                    related_families: vec![FamilyRef {
                        family: name.into(),
                    }],
                    pack: Pack { map_groups: vec![] },
                });
            }
        }
        Ok(catalog)
    }

    fn mode(&self, kind: ModeKind) -> Option<&Mode> {
        self.modes
            .iter()
            .find(|mode| mode.ruleset_group == kind && mode.wave.in_f2())
    }
}

impl Policy {
    pub(crate) fn load(cfg: &Config) -> io::Result<Option<Self>> {
        cfg.check_extended_activation()?;
        let visible = cfg.visible_map_groups.resolve()?;
        if cfg.create_overrides.is_empty() && cfg.runtime_overrides.is_empty() && visible.is_empty()
        {
            return Ok(None);
        }
        let catalog = Catalog::load()?;
        let rules = RuleTable::load(&catalog.modes)?;
        let create = cfg.create_overrides.resolve(&rules)?;
        let runtime = cfg.runtime_overrides.resolve(&rules)?;
        let CreateOverrides { free, manner } = create;
        if [free, manner]
            .into_iter()
            .flatten()
            .any(|kind| catalog.mode(kind).is_none())
        {
            return Err(io::Error::other("room create override is outside F2"));
        }
        if (free.is_some() && runtime.free.is_some())
            || (manner.is_some() && runtime.manner.is_some())
        {
            return Err(io::Error::other("F2 create and runtime overrides overlap"));
        }
        Ok(Some(Self {
            catalog,
            rules,
            visible,
            create,
            runtime,
        }))
    }

    fn override_for(&self, team: RoomTeamSelector) -> Option<ModeKind> {
        self.create.for_team(team)
    }

    pub(crate) fn has_create_override(&self, team: RoomTeamSelector) -> bool {
        self.override_for(team).is_some()
    }

    pub(crate) fn checks_native_start(&self) -> bool {
        self.create.free.is_some() || self.create.manner.is_some() || !self.visible.is_empty()
    }

    fn group_visible(&self, group: &Group) -> bool {
        MapGroup::from_locator(&group.locator).is_some_and(|id| self.visible.contains(&id))
    }

    fn validate_candidate_map(&self, kind: ModeKind, name: &[u8]) -> Result<(), &'static str> {
        let mode = self.catalog.mode(kind).ok_or("mode_outside_f2")?;
        if !mode.pack.map_groups.is_empty() {
            let mut visible = mode
                .pack
                .map_groups
                .iter()
                .filter(|g| self.group_visible(g))
                .peekable();
            if visible.peek().is_none() {
                return Err("f2_map_group_hidden");
            }
            // Only unique literal census matches qualify. In particular, no
            // expansion of ReBirthMapGroup's Tomb? and Cemetery? is known.
            if visible.any(|g| {
                g.exact_census_matches
                    .iter()
                    .flatten()
                    .any(|m| m.entry.as_bytes() == name)
            }) {
                return Ok(());
            }
        } else if mode.related_families.iter().any(|related| {
            self.catalog.families.iter().any(|family| {
                family.family == related.family
                    && family.maps.iter().any(|m| m.name.as_bytes() == name)
            })
        }) {
            return Ok(());
        }
        // Some related families have only counts in the evidence, not names.
        // Neither a name prefix nor a numeric suffix proves membership.
        Err("f2_map_not_confirmed_in_group_or_family")
    }

    pub(crate) fn check_start(
        &self,
        team: RoomTeamSelector,
        map: &Start121Map,
        selected: (u16, u32),
    ) -> Result<(), &'static str> {
        if (map.map_code, map.dynamic_index) != selected {
            return Err("start_map_selection_mismatch");
        }
        if TENTATIVE_MAP_FAMILIES
            .iter()
            .any(|(_, maps)| maps.iter().any(|name| name.as_bytes() == map.name))
        {
            return Err("f2_candidate_map_dispatch_unmeasured");
        }
        if let Some(kind) = self.override_for(team) {
            self.validate_candidate_map(kind, &map.name)?;
            if kind
                .evaluation_map_group()
                .is_some_and(|group| !self.visible.contains(&group))
            {
                return Err("f2_map_group_hidden");
            }
            return self.rules.check_start(kind);
        }
        if map.roomtype != 0 {
            return Err("start_roomtype_unmeasured");
        }
        // The evaluation presets use the managed roomtype=0 route, limited to
        // exact names in their respective catalog families.
        if let Some(kind) = self.runtime.for_team(team)
            && (kind.evaluation_map_group().is_some() || kind == ModeKind::Hamster)
        {
            self.validate_candidate_map(kind, &map.name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn room_map_groups_are_hidden_by_default_and_filter_only_exact_matches() {
        for (visible, ladder, revive) in [
            (
                json!([]),
                Err("f2_map_group_hidden"),
                Err("f2_map_group_hidden"),
            ),
            (
                json!(["LadderMapGroup"]),
                Ok(()),
                Err("f2_map_group_hidden"),
            ),
            (
                json!(["ReBirthMapGroup"]),
                Err("f2_map_group_hidden"),
                Err("f2_map_not_confirmed_in_group_or_family"),
            ),
        ] {
            let cfg = serde_json::from_value(json!({
                "visible_map_groups": visible,
                "create_overrides": {"free": "ladder"}
            }))
            .unwrap();
            let policy = Policy::load(&cfg).unwrap().unwrap();
            assert_eq!(
                policy.validate_candidate_map(ModeKind::Ladder, b"Factory07"),
                ladder
            );
            assert_eq!(
                policy.validate_candidate_map(ModeKind::Revive, b"Tomb12"),
                revive
            );
            assert!(
                policy
                    .validate_candidate_map(ModeKind::Revive, b"Tomb?")
                    .is_err()
            );
            assert!(
                policy
                    .validate_candidate_map(ModeKind::Ladder, b"Factory08")
                    .is_err()
            );
        }
    }

    #[test]
    fn room_mode_family_membership_uses_census_names_and_does_not_grant_start() {
        let cfg =
            serde_json::from_value(json!({"create_overrides": {"manner": "baseball"}})).unwrap();
        let policy = Policy::load(&cfg).unwrap().unwrap();
        assert!(
            policy
                .validate_candidate_map(ModeKind::Baseball, b"Baseball01")
                .is_ok()
        );
        for (mode, name) in [
            (ModeKind::Baseball, b"Baseball99".as_slice()),
            (ModeKind::Baseball, b"baseball01"),
            (ModeKind::ConfusedFight, b"ConfusedFight01"),
            (ModeKind::SnowballEvent, b"SnowyFields01"),
        ] {
            assert_eq!(
                policy.validate_candidate_map(mode, name),
                Err("f2_map_not_confirmed_in_group_or_family")
            );
        }
        let mut map = Start121Map {
            map_code: 900,
            name: b"Baseball01".to_vec(),
            dynamic_index: 3,
            roomtype: 0,
        };
        assert_eq!(
            policy.check_start(RoomTeamSelector::Manner, &map, (900, 3)),
            Err("f2_start_dispatch_unmeasured")
        );
        map.roomtype = 1;
        assert_eq!(
            policy.check_start(RoomTeamSelector::Manner, &map, (900, 3)),
            Err("f2_start_dispatch_unmeasured")
        );
    }

    #[test]
    fn room_mode_config_rejects_wire_vals_unknown_groups_and_f3_overrides() {
        for input in [
            json!({"create_overrides": {"free": 1}}),
            json!({"create_overrides": {"ladder": "ladder"}}),
            json!({"create_overrides": {"free": "made_up"}}),
            json!({"visible_map_groups": ["Factory"]}),
        ] {
            assert!(serde_json::from_value::<Config>(input).is_err());
        }
        let f3 = serde_json::from_value(json!({"create_overrides": {"free": "popcade"}})).unwrap();
        assert!(Policy::load(&f3).is_err());
        let empty = serde_json::from_value(json!({"visible_map_groups": []})).unwrap();
        assert!(Policy::load(&empty).unwrap().is_none());
    }
}
