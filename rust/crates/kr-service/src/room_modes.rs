//! F2-1 operator policy and static KR00967 catalog. These IDs have no wire
//! representation. Attr membership and candidate family links are only filters.
//! KR00967 static anchors: MapGroupVisible writer 0xE04CB0 and reader 0xE04DD0;
//! create dialogs 0xC541E0 / 0xC55AF0 (Ladder), 0xC8DF50 (Mission), and
//! roomtype 0x57F520. These do not establish a live F2 packet contract.
use kr_wire::kr00967::{RoomTeamSelector, Start121Map};
use serde::Deserialize;
use std::{collections::BTreeSet, io};

const CATALOG: &str = include_str!("../../../../docs/evidence/content/f2-mode-catalog.json");

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ModeKind {
    Airplane,
    Antarctic,
    Baseball,
    BigBattle,
    CoOp,
    ConfusedFight,
    #[serde(rename = "free_1v7")]
    Free1v7,
    Hedgehog,
    Ladder,
    LeaderCapture,
    Matchmaking,
    Popcade,
    Revive,
    SnowballEvent,
    SnowyChamp,
    SnowyFields,
    SuperFighter,
    Timebomb,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum MapGroup {
    #[serde(rename = "LadderMapGroup")]
    Ladder,
    #[serde(rename = "ReBirthMapGroup")]
    Rebirth,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    visible_map_groups: BTreeSet<MapGroup>,
    #[serde(default)]
    create_overrides: CreateOverrides,
}

/// Explicit operator presets for requests that use a measured team selector.
/// The native selector always keeps its original meaning and wire value.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateOverrides {
    free: Option<ModeKind>,
    manner: Option<ModeKind>,
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
    wave: String,
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
    maps: Vec<MapName>,
}

#[derive(Deserialize)]
struct MapName {
    name: String,
}

pub(crate) struct Policy {
    catalog: Catalog,
    visible: BTreeSet<MapGroup>,
    free: Option<ModeKind>,
    manner: Option<ModeKind>,
}

impl Catalog {
    fn mode(&self, kind: ModeKind) -> Option<&Mode> {
        self.modes
            .iter()
            .find(|mode| mode.ruleset_group == kind && matches!(mode.wave.as_str(), "F2" | "F2?"))
    }
}

impl Policy {
    pub(crate) fn load(cfg: &Config) -> io::Result<Option<Self>> {
        let CreateOverrides { free, manner } = cfg.create_overrides;
        if free.is_none() && manner.is_none() && cfg.visible_map_groups.is_empty() {
            return Ok(None);
        }
        let catalog: Catalog = serde_json::from_str(CATALOG).map_err(io::Error::other)?;
        if catalog.build != "KR00967" || catalog.schema != "kre.f2-mode-catalog.v1" {
            return Err(io::Error::other("unsupported F2 catalog build or schema"));
        }
        if [free, manner]
            .into_iter()
            .flatten()
            .any(|kind| catalog.mode(kind).is_none())
        {
            return Err(io::Error::other("room create override is outside F2"));
        }
        Ok(Some(Self {
            catalog,
            visible: cfg.visible_map_groups.clone(),
            free,
            manner,
        }))
    }

    fn override_for(&self, team: RoomTeamSelector) -> Option<ModeKind> {
        match team {
            RoomTeamSelector::Free => self.free,
            RoomTeamSelector::Manner => self.manner,
        }
    }

    pub(crate) fn has_create_override(&self, team: RoomTeamSelector) -> bool {
        self.override_for(team).is_some()
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
        if let Some(kind) = self.override_for(team) {
            self.validate_candidate_map(kind, &map.name)?;
            // The catalog has no measured roomtype or rules for any F2 mode.
            return Err("f2_start_dispatch_unmeasured");
        }
        if map.roomtype != 0 {
            return Err("start_roomtype_unmeasured");
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
