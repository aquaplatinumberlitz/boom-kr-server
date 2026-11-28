use kr_service::choco_chat::{
    F5CoupleRuntime, F5Message, F5NoteInbox, F5PlazaLimits, F5PlazaRuntime, F5PlazaZone,
    F5SocialError, F5SocialPolicy, F5SocialRuntimeSnapshot,
};
use serde_json::Value;
use std::{collections::BTreeMap, num::NonZeroU16};

fn plaza_policy() -> F5SocialPolicy {
    let mut plaza_zones = BTreeMap::new();
    for zone in [
        F5PlazaZone::LobbySquare,
        F5PlazaZone::LobbyCaffe,
        F5PlazaZone::LobbyClothing,
        F5PlazaZone::LobbyJewelry,
        F5PlazaZone::LobbyPet,
        F5PlazaZone::Square,
    ] {
        plaza_zones.insert(
            zone,
            F5PlazaLimits {
                max_occupants: NonZeroU16::new(2).expect("constant is non-zero"),
                seats: if zone == F5PlazaZone::LobbyCaffe {
                    1
                } else {
                    0
                },
            },
        );
    }
    F5SocialPolicy {
        note_inbox_cap: NonZeroU16::new(2),
        couple: true,
        plaza_zones,
        ..Default::default()
    }
}

#[test]
fn disabled_defaults_fail_closed() {
    let policy = F5SocialPolicy::default();
    assert_eq!(F5NoteInbox::new(&policy, 7), Err(F5SocialError::Disabled));
    assert_eq!(
        F5CoupleRuntime::new(&policy).pair(1, 2),
        Err(F5SocialError::Disabled)
    );
    assert_eq!(
        F5PlazaRuntime::new(&policy).occupants(F5PlazaZone::LobbySquare),
        Err(F5SocialError::Disabled)
    );
}

#[test]
fn inbox_bounds_and_read_delete() {
    let policy = plaza_policy();
    let mut inbox = F5NoteInbox::new(&policy, 2).unwrap();
    assert_eq!(inbox.enqueue(&policy, 1, F5Message::Utf8("hello")), Ok(5));
    assert_eq!(inbox.enqueue(&policy, 1, F5Message::Utf8("again")), Ok(5));
    assert_eq!(
        inbox.enqueue(&policy, 1, F5Message::Utf8("full")),
        Err(F5SocialError::InboxFull)
    );
    assert_eq!(
        inbox.enqueue(&policy, 2, F5Message::Utf8("self")),
        Err(F5SocialError::Chat(
            kr_service::choco_chat::ChocoChatError::TargetUnavailable
        ))
    );
    assert_eq!(inbox.unread, 2);
    inbox.mark_read(1).unwrap();
    assert_eq!(inbox.unread, 1);
    inbox.delete(1).unwrap();
    assert_eq!((inbox.messages, inbox.unread), (1, 1));
    inbox.delete_all();
    assert_eq!((inbox.messages, inbox.unread), (0, 0));
}

#[test]
fn couple_lifecycle_is_mutually_unique() {
    let policy = plaza_policy();
    let mut runtime = F5CoupleRuntime::new(&policy);
    runtime.pair(1, 2).unwrap();
    assert_eq!(runtime.partner(1), Some(2));
    assert_eq!(runtime.pair(1, 3), Err(F5SocialError::PartnerOccupied));
    runtime.equip_ring(1).unwrap();
    assert!(runtime.has_ring(2));
    runtime.divorce(2).unwrap();
    assert_eq!(runtime.couple_count(), 0);
    assert_eq!(runtime.divorce(2), Err(F5SocialError::NotPaired));
}

#[test]
fn plaza_presence_and_cafe_seating_are_bounded() {
    let policy = plaza_policy();
    let mut runtime = F5PlazaRuntime::new(&policy);
    runtime.enter(F5PlazaZone::LobbyCaffe, 1).unwrap();
    runtime.enter(F5PlazaZone::LobbyCaffe, 2).unwrap();
    assert_eq!(
        runtime.enter(F5PlazaZone::LobbyCaffe, 3),
        Err(F5SocialError::PlazaFull)
    );
    runtime.sit(F5PlazaZone::LobbyCaffe, 1, 0).unwrap();
    assert_eq!(
        runtime.sit(F5PlazaZone::LobbyCaffe, 2, 0),
        Err(F5SocialError::SeatUnavailable)
    );
    runtime.leave(F5PlazaZone::LobbyCaffe, 1).unwrap();
    assert_eq!(runtime.seated(F5PlazaZone::LobbyCaffe), Ok(0));
    runtime.sit(F5PlazaZone::LobbyCaffe, 2, 0).unwrap();
    assert_eq!(runtime.occupants(F5PlazaZone::LobbyCaffe), Ok(1));
}

#[test]
fn snapshot_serializes_counts_without_private_state() {
    let policy = plaza_policy();
    let mut inbox = F5NoteInbox::new(&policy, 2).unwrap();
    inbox.enqueue(&policy, 1, F5Message::Utf8("hello")).unwrap();
    let mut couple = F5CoupleRuntime::new(&policy);
    couple.pair(1, 2).unwrap();
    couple.equip_ring(1).unwrap();
    let mut plaza = F5PlazaRuntime::new(&policy);
    plaza.enter(F5PlazaZone::LobbyCaffe, 1).unwrap();
    plaza.sit(F5PlazaZone::LobbyCaffe, 1, 0).unwrap();
    let snapshot = F5SocialRuntimeSnapshot::new(&policy, &[inbox], &couple, &plaza);
    assert_eq!(snapshot.couples, 1);
    assert_eq!(snapshot.ringed_couples, 1);
    assert_eq!(snapshot.unread_notes, 1);
    let value: Value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(value["plaza_occupants"], 1);
    assert!(value.get("sender_id").is_none());
}
