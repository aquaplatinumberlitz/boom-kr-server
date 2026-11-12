// Inputs retained from native KR00967 controls: CP282 standard Start and
// m7e-start-stat-probe-r2 reversed stat keys, run82b7c957...; no generated oracle.
use kr_wire::kr00967::{StartPlayer, start121_players};
fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
const CONTROL: &str = "5a96ca53ff96ca368d96ca539596ca539596ca539596ca5395a6f3539597dc539596ca539eff9c2bd0f1ab3ff996fb63f096ca539596ca519596ca539596ca53959635539569ca53959635ac6a96ca539596ca539597cb539496ca529596ca536a6935ac9596ca536a96ca5395a5695a22a55aa56996ca539396ca528da5695a5a";
const REVERSED: &str = "5a96ca53ff96ca368d96ca539596ca539596ca539596ca5395a6f3539597dc539596ca539eff9c2bd0f1ab3ff996fb63f096ca539596ca519596ca539596ca52959635539569ca53959635ac6a96ca539596ca539596cb539496ca529596ca536a6935ac9596ca536a96ca5395a5695a22a55aa56996ca539396ca528da5695a5a";

#[test]
fn explicit_character_keys_reproduce_independent_native_control_and_reversal() {
    let control = hex(CONTROL);
    let players = [
        StartPlayer {
            slot: 0,
            spawn: 0,
            character: 0,
            team: 0,
        },
        StartPlayer {
            slot: 1,
            spawn: 1,
            character: 1,
            team: 1,
        },
    ];
    assert_eq!(start121_players(&control, &players).unwrap(), control);
    let mut reversed = players;
    reversed[0].character = 1;
    reversed[1].character = 0;
    assert_eq!(
        start121_players(&control, &reversed).unwrap(),
        hex(REVERSED)
    );
    reversed.swap(0, 1); // Membership order is not room slot order.
    assert_eq!(
        start121_players(&control, &reversed).unwrap(),
        hex(REVERSED)
    );
}
#[test]
fn invalid_duplicate_fields_and_truncated_source_fail_without_emitting_a_start() {
    let control = hex(CONTROL);
    let players = [
        StartPlayer {
            slot: 0,
            spawn: 0,
            character: 0,
            team: 0,
        },
        StartPlayer {
            slot: 1,
            spawn: 1,
            character: 1,
            team: 1,
        },
    ];
    for n in 0..control.len() {
        assert!(start121_players(&control[..n], &players).is_err());
    }
    assert!(start121_players(&control, &[]).is_err());
    assert!(start121_players(&control, &players[..1]).is_err());
    for invalid in [
        StartPlayer {
            slot: 0,
            ..players[1]
        },
        StartPlayer {
            spawn: 0,
            ..players[1]
        },
        StartPlayer {
            slot: 8,
            ..players[1]
        },
        StartPlayer {
            spawn: 8,
            ..players[1]
        },
        StartPlayer {
            character: 0x2000,
            ..players[1]
        },
        StartPlayer {
            team: 32,
            ..players[1]
        },
    ] {
        assert!(start121_players(&control, &[players[0], invalid]).is_err());
    }
}
