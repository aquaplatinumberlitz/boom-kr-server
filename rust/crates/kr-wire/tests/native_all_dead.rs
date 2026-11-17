// CP083 actually received175, A-rank175-all-dead-r2-172.bin, seq172.
// Captured bytes are the oracle; no serializer-produced golden.
use kr_wire::kr00967::rank175_all_dead_players;
#[test]
fn all_dead_rank_rows_match_independent_native_two_player_capture() {
    let retained = "96ca52bf96cb768d96ca539596ca539569ca539596ca539596cb539594ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca529596c8539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca5395a569";
    let native: Vec<u8> = retained
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(rank175_all_dead_players(&[(0, 0), (1, 3)]).unwrap(), native);
    for roster in [
        vec![(0, 0)],
        vec![(0, 0), (0, 3)],
        vec![(0, 0), (8, 3)],
        vec![(0, 0), (1, 0)],
    ] {
        assert!(rank175_all_dead_players(&roster).is_err());
    }
}
