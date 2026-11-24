use super::*;

#[test]
fn map_option_rejects_bomb118_without_allocating_or_dropping_members() {
    for disabled in [true, false] {
        let shared = registry_foundation_tests::empty(0);
        let mut a = Session::open(&shared, 1, 11).unwrap();
        let mut b = Session::open(&shared, 2, 22).unwrap();
        let (first, peer, round) = {
            let mut runtime = shared.lock().unwrap();
            let model = &mut runtime.model;
            model.terrain = Some(if disabled {
                crate::terrain::tests::lovely_room_map()
            } else {
                crate::terrain::tests::map()
            });
            let first = model.enter(1, 11, true).unwrap();
            let peer = model.enter(2, 22, false).unwrap();
            model.setup(first).unwrap();
            model.setup(peer).unwrap();
            model.ready(peer).unwrap();
            let round = model.begin(first).unwrap();
            model.ack(first, round).unwrap();
            model.ack(peer, round).unwrap();
            (first, peer, round)
        };
        let request: Vec<_> = [
            (u64::from(crate::room::wire_key(round).unwrap()), Width::U32),
            (0, Width::U8),
            (0, Width::U8),
            (1, Width::U32),
            (3, Width::U8),
            (3, Width::U8),
            (1, Width::U8),
            (0, Width::U8),
            (0, Width::U8),
            (0, Width::U8),
            (1, Width::U8),
            (1, Width::U32),
            (0, Width::U8),
        ]
        .into_iter()
        .flat_map(|(value, width)| kr_wire::write_uint(value, width).unwrap())
        .collect();
        let result = a.packet(118, &request).unwrap().bomb.unwrap();
        if disabled {
            assert_eq!(result.reason, "map_bombs_disabled");
            assert!(!result.accepted && result.id.is_none());
            assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());
            let again = a.packet(118, &request).unwrap().bomb.unwrap();
            assert_eq!(again.reason, "map_bombs_disabled");
            assert!(!shared.lock().unwrap().model.bombs.pending());
        } else {
            assert!(result.accepted);
            assert_eq!(result.id, Some(1));
            for inbox in [&mut a.inbox, &mut b.inbox] {
                assert_eq!(inbox.try_recv().unwrap().packets[0].1.opcode, 159);
                assert_eq!(inbox.try_recv().unwrap().packets[0].1.opcode, 158);
            }
        }
        assert!(a.channel_current() && b.channel_current());
        let runtime = shared.lock().unwrap();
        assert!(runtime.model.current(first) && runtime.model.current(peer));
        assert_eq!(runtime.model.phase, Phase::Armed { round });
    }
}
