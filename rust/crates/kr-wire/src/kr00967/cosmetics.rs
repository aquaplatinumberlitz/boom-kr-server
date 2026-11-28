//! F4 visual and catalog slot matrix for KR 00967.
//! Source: docs/evidence/content/f4-cosmetics-projection-spec.json.
//!
//! The measured routes carry the base BomberAttr character key. They do not
//! map an equipped SKU to that key. Use the existing room codecs for those
//! routes; this module does not build packets or grant equipment entitlement.

#![deny(unsafe_code)]

use super::inventory::{
    Inventory219, Inventory224, Inventory225, Inventory313, Inventory314, Inventory315,
    Inventory316, Inventory317, Inventory566,
};
use std::{collections::BTreeSet, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogCategory {
    Character,
    Collection,
    Consumable,
    Decoration,
    Emotion,
}

impl CatalogCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Character => "character",
            Self::Collection => "collection",
            Self::Consumable => "consumable",
            Self::Decoration => "decoration",
            Self::Emotion => "emotion",
        }
    }
}

/// Provenance of an internal catalog slot token, independent of wire status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotAssignment {
    Preset(&'static str),
    Proposed(&'static str),
    Conditional(&'static str),
    Fixture {
        token: &'static str,
        input: &'static str,
    },
    Unassigned,
}

impl SlotAssignment {
    pub const fn token(self) -> Option<&'static str> {
        match self {
            Self::Preset(token)
            | Self::Proposed(token)
            | Self::Conditional(token)
            | Self::Fixture { token, .. } => Some(token),
            Self::Unassigned => None,
        }
    }

    pub const fn evidence_input(self) -> Option<&'static str> {
        match self {
            Self::Preset(_) => Some("preset"),
            Self::Proposed(_) | Self::Conditional(_) => Some("bc_summary"),
            Self::Fixture { input, .. } => Some(input),
            Self::Unassigned => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetSpec {
    pub catalog_category: Option<CatalogCategory>,
    pub source_group: Option<&'static str>,
    /// Count of pack entries, distinct from the number of exported SKUs.
    pub pack_entry_count: Option<u32>,
    pub slot: SlotAssignment,
    /// Schema 8 design evidence; an empty list means persistence is unknown.
    pub persistence_anchors: &'static [&'static str],
    /// Comparison evidence only; these anchors do not qualify as projections.
    pub context_anchors: &'static [&'static str],
}

impl TargetSpec {
    const fn new(
        catalog_category: Option<CatalogCategory>,
        source_group: Option<&'static str>,
        pack_entry_count: Option<u32>,
        slot: SlotAssignment,
        persistence_anchors: &'static [&'static str],
    ) -> Self {
        Self {
            catalog_category,
            source_group,
            pack_entry_count,
            slot,
            persistence_anchors,
            context_anchors: &[],
        }
    }
}

const OWNED: &[&str] = &["db_inventory"];
const EQUIPPED: &[&str] = &["db_inventory", "db_equipped"];
const CONSUMABLE: &[&str] = &["db_inventory", "db_consumption"];
const STACK_SLOT: &[&str] = &["db_inventory", "db_equipped", "db_consumption"];
const CHARACTER: &[&str] = &["db_inventory", "db_equipped", "db_character_preference"];

macro_rules! targets {
    ($($target:ident => $name:literal, $row:expr;)*) => {
        /// A row of the committed matrix, including categories with no known SKU or slot.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum CosmeticTarget { $($target),* }

        impl CosmeticTarget {
            pub const ALL: &'static [Self] = &[$(Self::$target),*];

            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$target => $name),* }
            }

            pub const fn spec(self) -> TargetSpec {
                use CatalogCategory as C;
                use SlotAssignment::{Conditional, Fixture, Preset, Proposed, Unassigned};
                match self { $(Self::$target => $row),* }
            }
        }
    };
}

targets! {
    Avatar => "avatar",
        TargetSpec::new(Some(C::Decoration), Some("CharTE/AvatarDeco"), Some(7797), Proposed("cosmetic.avatar"), EQUIPPED);
    Backup => "backup",
        TargetSpec::new(Some(C::Consumable), None, None, Fixture { token: "backup", input: "consumption_fixtures" }, STACK_SLOT);
    Balloon => "balloon", TargetSpec::new(None, None, None, Unassigned, &[]);
    Bomb => "bomb",
        TargetSpec::new(Some(C::Decoration), Some("CharTE/BombDeco"), Some(594), Proposed("cosmetic.bomb"), EQUIPPED);
    CharDecoration => "char_decoration",
        TargetSpec::new(Some(C::Decoration), Some("CharTE/CharDeco"), Some(3896), Proposed("cosmetic.char"), EQUIPPED);
    Character => "character",
        TargetSpec::new(Some(C::Character), Some("FxContents/Attr/BomberAttr"), None, Preset("character"), CHARACTER);
    Collection => "collection", TargetSpec::new(Some(C::Collection), Some("fx/Pet"), None, Unassigned, OWNED);
    CollectionPanel => "collection_panel",
        TargetSpec::new(None, Some("FxContents/ItemCollection"), Some(27), Unassigned, &[]);
    Consumable => "consumable",
        TargetSpec::new(Some(C::Consumable), Some("FxContents/Attr/ItemAttr"), None, Unassigned, CONSUMABLE);
    Effect => "effect", TargetSpec::new(None, None, None, Unassigned, &[]);
    Emote => "emote",
        TargetSpec::new(Some(C::Emotion), None, None, Fixture { token: "emote", input: "shop_fixtures" }, EQUIPPED);
    EmoteSlot => "emote_slot", TargetSpec::new(Some(C::Decoration), Some("EmoTE/Slot"), None, Unassigned, OWNED);
    Emoticon => "emoticon", TargetSpec::new(Some(C::Emotion), Some("EmoTE/Character"), Some(1479), Unassigned, OWNED);
    EmoticonEffect => "emoticon_effect", TargetSpec::new(None, Some("fx/Emoticon"), Some(52), Unassigned, &[]);
    Fanfare => "fanfare", TargetSpec::new(Some(C::Emotion), Some("EmoTE/Fanfare"), None, Unassigned, OWNED);
    Fishing => "fishing", TargetSpec::new(Some(C::Decoration), Some("CharTE/FishingDeco"), None, Unassigned, OWNED);
    Flag => "flag", TargetSpec::new(None, None, None, Unassigned, &[]);
    Grade => "grade",
        TargetSpec::new(Some(C::Decoration), Some("CharTE/GradeDeco"), Some(211), Conditional("cosmetic.grade"), EQUIPPED);
    Hat => "hat", TargetSpec::new(None, None, None, Fixture { token: "head", input: "runtime_fixtures" }, EQUIPPED);
    Id => "id", TargetSpec::new(Some(C::Decoration), Some("EmoTE/IDDeco"), Some(3176), Proposed("cosmetic.id"), EQUIPPED);
    Ladder => "ladder", TargetSpec::new(Some(C::Decoration), Some("CharTE/LadderDeco"), None, Unassigned, OWNED);
    Outfit => "outfit", TargetSpec::new(None, None, None, Unassigned, &[]);
    Quick => "quick",
        TargetSpec::new(Some(C::Consumable), None, None, Fixture { token: "quick", input: "runtime_fixtures" }, STACK_SLOT);
    Ring => "ring", TargetSpec::new(None, None, None, Unassigned, &[]);
    Vehicle => "vehicle", TargetSpec {
        context_anchors: &["f1_vehicle_pickup"],
        ..TargetSpec::new(None, Some("FxContents/Attr/ItemAttr: Ship/*"), None, Unassigned, &[])
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionSurface {
    KrWire,
    Own291,
    Start121,
    RoomSlot,
}

impl ProjectionSurface {
    pub const ALL: [Self; 4] = [Self::KrWire, Self::Own291, Self::Start121, Self::RoomSlot];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KrWire => "kr_wire",
            Self::Own291 => "own291",
            Self::Start121 => "start121",
            Self::RoomSlot => "room_slot",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionStatus {
    Measured,
    StaticCandidate,
    Unmeasured,
}

/// Site in the unpacked KR 00967 image, given as an IDA virtual address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F4AnchorSite {
    ShopStageDispatcher(u32),
    Function(u32),
    /// Client handler or packet consumer for an incoming opcode.
    ServerToClientHandler(u32),
    /// Client site that builds an outgoing packet.
    ClientToServerBuilder(u32),
}

/// Evidence for an F4 site. An anchor does not enable a codec or map a SKU
/// to a cosmetic slot. Measured entries cover only the fields in their capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F4ShopAnchor {
    pub name: &'static str,
    /// `None` for dispatchers and helpers without an opcode of their own.
    pub opcode: Option<u32>,
    pub site: F4AnchorSite,
    pub status: ProjectionStatus,
    /// Repository-relative evidence or protocol reference.
    pub evidence: &'static str,
}

impl F4ShopAnchor {
    const fn static_site(
        name: &'static str,
        opcode: Option<u32>,
        site: F4AnchorSite,
        evidence: &'static str,
    ) -> Self {
        Self {
            name,
            opcode,
            site,
            status: ProjectionStatus::StaticCandidate,
            evidence,
        }
    }
}

const F4_INVENTORY_W2: &str = "docs/evidence/content/f4-w2-ida-deepdive.json";
const F4_INVENTORY_W3: &str = "docs/evidence/content/f4-w3-ida-records.json";

/// Static provenance for shop, inventory and related account packet sites.
pub static F4_SHOP_ANCHORS: &[F4ShopAnchor] = &[
    F4ShopAnchor::static_site(
        "CShopStageNew",
        None,
        F4AnchorSite::ShopStageDispatcher(0x1072B10),
        "docs/protocol/f4-inventory-cosmetics.md#f4b-inventory-contract",
    ),
    F4ShopAnchor::static_site(
        "sub_11CE930",
        None,
        F4AnchorSite::Function(0x11CE930),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_125D190",
        None,
        F4AnchorSite::Function(0x125D190),
        "docs/evidence/content/f4-w4-ida-gates.json",
    ),
    F4ShopAnchor::static_site(
        "sub_AE6330",
        None,
        F4AnchorSite::Function(0xAE6330),
        "docs/evidence/content/f4-w6-ida-own291-and-hud.json",
    ),
    F4ShopAnchor::static_site(
        "sub_D6E890",
        Some(20),
        F4AnchorSite::ServerToClientHandler(0xD6E890),
        "docs/evidence/content/f4-w5-ida-ui-and-exp.json",
    ),
    F4ShopAnchor {
        name: "lucci135",
        opcode: Some(135),
        site: F4AnchorSite::ServerToClientHandler(0xD6CED0),
        status: ProjectionStatus::Measured,
        evidence: "docs/protocol/life-results.md#f4a-lucci135-wine-accept",
    },
    F4ShopAnchor::static_site(
        "sub_11CEA10",
        Some(Inventory219::OPCODE),
        F4AnchorSite::ServerToClientHandler(0x11CEA10),
        F4_INVENTORY_W2,
    ),
    F4ShopAnchor::static_site(
        "sub_11CE4C0",
        Some(Inventory224::OPCODE),
        F4AnchorSite::ServerToClientHandler(0x11CE4C0),
        F4_INVENTORY_W2,
    ),
    F4ShopAnchor::static_site(
        "sub_11CE6A0",
        Some(Inventory225::OPCODE),
        F4AnchorSite::ServerToClientHandler(0x11CE6A0),
        F4_INVENTORY_W2,
    ),
    F4ShopAnchor {
        name: "own291",
        opcode: Some(291),
        site: F4AnchorSite::ServerToClientHandler(0xD73BB0),
        status: ProjectionStatus::Measured,
        evidence: "docs/protocol/life-results.md#f4a-own291-level-wine-accept",
    },
    F4ShopAnchor::static_site(
        "sub_D779C0",
        Some(Inventory313::OPCODE),
        F4AnchorSite::ServerToClientHandler(0xD779C0),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_D77BD0",
        Some(Inventory314::OPCODE),
        F4AnchorSite::ServerToClientHandler(0xD77BD0),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_D77D60",
        Some(Inventory315::OPCODE),
        F4AnchorSite::ServerToClientHandler(0xD77D60),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_D77EF0",
        Some(Inventory316::OPCODE),
        F4AnchorSite::ServerToClientHandler(0xD77EF0),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_D84410",
        Some(Inventory317::OPCODE),
        F4AnchorSite::ServerToClientHandler(0xD84410),
        F4_INVENTORY_W3,
    ),
    F4ShopAnchor::static_site(
        "sub_11AECE0",
        Some(Inventory566::OPCODE),
        F4AnchorSite::ClientToServerBuilder(0x11AFA5D),
        F4_INVENTORY_W2,
    ),
];

/// Looks up opcode provenance in either direction. A match does not imply a
/// usable shop or cosmetic projection route.
pub fn f4_shop_anchor_for_opcode(opcode: u32) -> Option<&'static F4ShopAnchor> {
    F4_SHOP_ANCHORS
        .iter()
        .find(|anchor| anchor.opcode == Some(opcode))
}

pub const UNMEASURED_WIRE_GAP: &str = "unmeasured_wire_projection";

/// Applies to every row. Static inventory layouts exist, but no positions or
/// routes have been mapped to cosmetic slots by native capture.
pub const INVENTORY_CAPTURE_GAP: &str = "in_game_219_224_225_313_317_566_pending_capture";
pub const INVENTORY_LAYOUT_STATUS: ProjectionStatus = ProjectionStatus::StaticCandidate;

/// Native anchors for the base character key; use the existing KR 00967 codecs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasuredProjection {
    /// C→S character choice: `local_character92`.
    Character92,
    /// S→C roster and slot state: `player106_character` and `slot110`.
    CharacterRoomSlot,
    /// S→C Start roster: `start121_players` and `start121_characters`.
    CharacterStart121,
}

impl MeasuredProjection {
    pub const fn anchor(self) -> &'static str {
        match self {
            Self::Character92 => "character92",
            Self::CharacterRoomSlot => "character_room_slot",
            Self::CharacterStart121 => "character_start121",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    Unmeasured {
        target: CosmeticTarget,
        surface: ProjectionSurface,
    },
    SlotMismatch {
        target: CosmeticTarget,
        surface: ProjectionSurface,
    },
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (reason, target, surface) = match *self {
            Self::Unmeasured { target, surface } => (UNMEASURED_WIRE_GAP, target, surface),
            Self::SlotMismatch { target, surface } => (
                "slot token does not match the measured target",
                target,
                surface,
            ),
        };
        write!(f, "{} / {}: {reason}", target.as_str(), surface.as_str())
    }
}

impl std::error::Error for ProjectionError {}

impl CosmeticTarget {
    /// Confirms an appearance route, never a mapping from an equipped SKU.
    pub const fn validate_projection(
        self,
        surface: ProjectionSurface,
    ) -> Result<MeasuredProjection, ProjectionError> {
        use MeasuredProjection::*;
        use ProjectionSurface::*;
        match (self, surface) {
            (Self::Character, KrWire) => Ok(Character92),
            (Self::Character, Start121) => Ok(CharacterStart121),
            (Self::Character, RoomSlot) => Ok(CharacterRoomSlot),
            _ => Err(ProjectionError::Unmeasured {
                target: self,
                surface,
            }),
        }
    }

    pub const fn projection_status(self, surface: ProjectionSurface) -> ProjectionStatus {
        if self.validate_projection(surface).is_ok() {
            ProjectionStatus::Measured
        } else {
            ProjectionStatus::Unmeasured
        }
    }

    /// Check both the measured route and the exact internal token for this
    /// target. Proposed and fixture slots cannot unlock a route.
    pub fn validate_slot_projection(
        self,
        slot: &str,
        surface: ProjectionSurface,
    ) -> Result<MeasuredProjection, ProjectionError> {
        let measured = self.validate_projection(surface)?;
        if self.spec().slot.token() != Some(slot) {
            return Err(ProjectionError::SlotMismatch {
                target: self,
                surface,
            });
        }
        Ok(measured)
    }

    /// Summary in matrix column order. This is for inspection, so unmeasured
    /// cells are omitted. Use the validators when a specific cell is required.
    pub fn measured_surfaces(
        self,
    ) -> impl Iterator<Item = (ProjectionSurface, MeasuredProjection)> {
        ProjectionSurface::ALL
            .into_iter()
            .filter_map(move |surface| {
                self.validate_projection(surface)
                    .ok()
                    .map(|projection| (surface, projection))
            })
    }
}

/// Two equipment rows for the same player conflict exactly when their internal
/// slot tokens agree (Schema 8 retains the `player_equipped` primary key).
pub fn slots_conflict(left: &str, right: &str) -> bool {
    left == right
}

/// Check mutual exclusion for one player's catalog slots. Returns the first
/// repeated token in input order. Distinct slots can reference the same SKU;
/// the spec supplies no cross-slot exclusion rules.
pub fn validate_mutual_exclusion<'a>(slots: &[&'a str]) -> Result<(), &'a str> {
    let mut seen = BTreeSet::new();
    for &slot in slots {
        if !seen.insert(slot) {
            return Err(slot);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str =
        include_str!("../../../../../docs/evidence/content/f4-cosmetics-projection-spec.json");

    // The committed fixture uses the generator's two-space indentation. Read
    // only the matrix fields we model, without adding a crate dependency.
    fn field<'a>(object: &'a str, depth: usize, name: &str) -> &'a str {
        let indent = "  ".repeat(depth);
        let key = format!("\n{indent}\"{name}\": ");
        let (_, value) = object
            .split_once(&key)
            .unwrap_or_else(|| panic!("missing {name}"));
        if let Some(close) = if value.starts_with("{\n") {
            Some('}')
        } else if value.starts_with("[\n") {
            Some(']')
        } else {
            None
        } {
            let end = format!("\n{indent}{close}");
            &value[..value.find(&end).unwrap() + end.len()]
        } else {
            value.lines().next().unwrap().trim_end_matches(',')
        }
    }

    fn keys(object: &str, depth: usize) -> Vec<&str> {
        let prefix = format!("{}\"", "  ".repeat(depth));
        object
            .lines()
            .filter_map(|line| {
                line.strip_prefix(&prefix)?
                    .split_once("\": ")
                    .map(|(key, _)| key)
            })
            .collect()
    }

    fn string(value: &str) -> Option<&str> {
        if value == "null" {
            None
        } else {
            Some(value.strip_prefix('"').unwrap().strip_suffix('"').unwrap())
        }
    }

    fn strings(array: &str) -> Vec<&str> {
        let inner = array.strip_prefix('[').unwrap().strip_suffix(']').unwrap();
        if inner.is_empty() {
            vec![]
        } else {
            inner
                .split(',')
                .map(|entry| string(entry.trim()).unwrap())
                .collect()
        }
    }

    #[test]
    fn catalog_slots_and_context_match_every_committed_target() {
        let targets = field(SPEC, 1, "targets");
        assert_eq!(
            keys(targets, 2),
            CosmeticTarget::ALL
                .iter()
                .map(|target| target.as_str())
                .collect::<Vec<_>>()
        );
        let defaults = field(field(SPEC, 1, "catalog"), 2, "default_equip_slots");
        let mut categories = BTreeSet::new();
        for &target in CosmeticTarget::ALL {
            let row = field(targets, 2, target.as_str());
            let spec = target.spec();
            let category = spec.catalog_category.map(CatalogCategory::as_str);
            assert_eq!(
                category,
                string(field(row, 3, "catalog_category")),
                "{target:?}"
            );
            assert_eq!(
                spec.source_group,
                string(field(row, 3, "source_group")),
                "{target:?}"
            );
            assert_eq!(
                spec.pack_entry_count,
                field(row, 3, "pack_entry_count").parse().ok(),
                "{target:?}"
            );

            let slot = field(row, 3, "slot");
            assert_eq!(
                spec.slot.token(),
                string(field(slot, 4, "name")),
                "{target:?}"
            );
            assert_eq!(
                spec.slot.evidence_input(),
                string(field(slot, 4, "input")),
                "{target:?}"
            );
            let (state, preset) = match spec.slot {
                SlotAssignment::Preset(token) => ("preset", Some(token)),
                SlotAssignment::Proposed(_) => ("proposed", None),
                SlotAssignment::Conditional(_) => ("conditional", None),
                SlotAssignment::Fixture { .. } => ("fixture", None),
                SlotAssignment::Unassigned => ("unassigned", None),
            };
            assert_eq!(Some(state), string(field(slot, 4, "state")), "{target:?}");
            if let Some(category) = category {
                categories.insert(category);
                assert_eq!(preset, string(field(defaults, 3, category)), "{target:?}");
            }

            let persistence = field(row, 3, "schema8_persistence");
            assert_eq!(
                spec.persistence_anchors,
                strings(field(persistence, 4, "anchors")),
                "{target:?}"
            );
            let status = if spec.persistence_anchors.is_empty() {
                "unknown"
            } else {
                "design"
            };
            assert_eq!(
                Some(status),
                string(field(persistence, 4, "status")),
                "{target:?}"
            );
            assert_eq!(
                spec.context_anchors,
                strings(field(row, 3, "context_anchors")),
                "{target:?}"
            );
            assert_eq!(
                Some(INVENTORY_CAPTURE_GAP),
                string(field(row, 3, "inventory_capture_gap")),
                "{target:?}"
            );
        }
        assert_eq!(
            keys(defaults, 3),
            categories.into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn all_projection_cells_and_the_measured_summary_match_committed_evidence() {
        let targets = field(SPEC, 1, "targets");
        let anchors = field(SPEC, 1, "anchors");
        let gaps = field(SPEC, 1, "gaps");
        let inventory = field(gaps, 2, INVENTORY_CAPTURE_GAP);
        assert_eq!(
            string(field(inventory, 3, "static_source")),
            Some("kr_static")
        );
        assert_eq!(string(field(inventory, 3, "status")), Some("unknown"));
        assert_eq!(
            string(field(field(inventory, 3, "s2c_225"), 4, "slot_mapping")),
            Some("unknown")
        );
        assert_eq!(INVENTORY_LAYOUT_STATUS, ProjectionStatus::StaticCandidate);

        for &target in CosmeticTarget::ALL {
            let projection = field(field(targets, 2, target.as_str()), 3, "projection");
            let mut columns = ProjectionSurface::ALL.map(ProjectionSurface::as_str);
            columns.sort_unstable();
            assert_eq!(keys(projection, 4), columns, "{target:?}");

            let mut expected_summary = Vec::new();
            for surface in ProjectionSurface::ALL {
                let cell = field(projection, 4, surface.as_str());
                let result = target.validate_projection(surface);
                let slot = target.spec().slot.token().unwrap_or("character");
                match string(field(cell, 5, "status")) {
                    Some("confirmed") => {
                        let name = string(field(cell, 5, "anchor")).unwrap();
                        let anchor = field(anchors, 2, name);
                        assert_eq!(string(field(anchor, 3, "source")), Some("kr_runtime"));
                        assert_eq!(string(field(anchor, 3, "status")), Some("confirmed"));
                        assert_eq!(
                            target.projection_status(surface),
                            ProjectionStatus::Measured
                        );
                        assert_eq!(result.unwrap().anchor(), name);
                        assert_eq!(target.validate_slot_projection(slot, surface), result);
                        expected_summary.push((surface, name));
                    }
                    Some("unknown") => {
                        assert_eq!(string(field(cell, 5, "gap")), Some(UNMEASURED_WIRE_GAP));
                        let error = Err(ProjectionError::Unmeasured { target, surface });
                        assert_eq!(
                            target.projection_status(surface),
                            ProjectionStatus::Unmeasured
                        );
                        assert_eq!(result, error);
                        assert_eq!(target.validate_slot_projection(slot, surface), error);
                    }
                    status => panic!("unrecognized status for {target:?}/{surface:?}: {status:?}"),
                }
            }
            assert_eq!(
                target
                    .measured_surfaces()
                    .map(|(surface, projection)| (surface, projection.anchor()))
                    .collect::<Vec<_>>(),
                expected_summary
            );
        }
    }

    #[test]
    fn wrong_tokens_cannot_use_a_measured_character_route() {
        let target = CosmeticTarget::Character;
        for (surface, _) in target.measured_surfaces() {
            for wrong in ["", "Character", "character ", "head", "cosmetic.char"] {
                assert_eq!(
                    target.validate_slot_projection(wrong, surface),
                    Err(ProjectionError::SlotMismatch { target, surface })
                );
            }
        }
    }

    #[test]
    fn mutual_exclusion_is_per_exact_token_and_reports_first_repeat() {
        let equipped = field(field(SPEC, 1, "anchors"), 2, "db_equipped");
        assert_eq!(strings(field(equipped, 3, "key")), ["player_id", "slot"]);
        assert!(slots_conflict("head", "head"));
        assert!(!slots_conflict("head", "Head"));
        assert!(!slots_conflict("character", "cosmetic.char"));
        assert_eq!(validate_mutual_exclusion(&[]), Ok(()));
        assert_eq!(validate_mutual_exclusion(&["backup", "quick"]), Ok(()));
        assert_eq!(
            validate_mutual_exclusion(&["head", "quick", "quick", "head"]),
            Err("quick")
        );
    }
}
