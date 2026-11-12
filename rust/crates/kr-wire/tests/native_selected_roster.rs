// Retained native CP282 rank body; serializer output is not the oracle.
use kr_wire::kr00967::rank175_players;
fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn player_rank_matches_independent_two_client_native_body() {
    let native = hex(
        "96ca52bf96cb768d96ca539596ca539569ca539596ca539596cb539594ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca529596cb539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca5395a569",
    );
    assert_eq!(rank175_players(&[(0, 0), (1, 1)], 1).unwrap(), native);
}
