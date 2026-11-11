//! Explicit environment selection for controlled Internet measurements.
//! Socket admission still uses the exact configured peer allowlist. This does
//! not add session-based transport binding, NAT discovery or endpoint learning.
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkScope {
    #[default]
    PrivateLab,
    ControlledInternet,
}

impl NetworkScope {
    pub(crate) fn permits(self, bind: Ipv4Addr, peers: &[Ipv4Addr], bounded_lobby: bool) -> bool {
        if !(bind.is_loopback() || (bounded_lobby && bind.is_private())) || peers.is_empty() {
            return false;
        }
        match self {
            Self::PrivateLab => peers
                .iter()
                .all(|ip| ip.is_loopback() || (bounded_lobby && ip.is_private())),
            Self::ControlledInternet => {
                bounded_lobby
                    && peers
                        .iter()
                        .all(|ip| !ip.is_unspecified() && !ip.is_multicast() && !ip.is_broadcast())
            }
        }
    }
}

/// Operator admission budgets, independent of the number of pinned profiles.
/// These are resource policy bounds, not native KR limits or measured capacity.
pub(crate) fn admission_budget(connections: usize, peers: &[Ipv4Addr]) -> bool {
    (1..=1024).contains(&connections) && (1..=256).contains(&peers.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_default_keeps_existing_bind_and_peer_boundaries() {
        let scope = NetworkScope::default();
        assert!(scope.permits(Ipv4Addr::LOCALHOST, &[Ipv4Addr::LOCALHOST], false));
        let private = Ipv4Addr::new(10, 0, 0, 2);
        assert!(!scope.permits(private, &[private], false));
        assert!(scope.permits(private, &[private], true));
        assert!(!scope.permits(private, &[Ipv4Addr::new(198, 51, 100, 7)], true));
        assert!(!scope.permits(Ipv4Addr::UNSPECIFIED, &[private], true));
    }

    #[test]
    fn internet_scope_requires_bounded_lobby_and_explicit_unicast_peers() {
        let scope = NetworkScope::ControlledInternet;
        let bind = Ipv4Addr::new(10, 0, 0, 2);
        let peers = [
            Ipv4Addr::new(198, 51, 100, 7),
            Ipv4Addr::new(203, 0, 113, 9),
        ];
        assert!(scope.permits(bind, &peers, true));
        assert!(!scope.permits(bind, &peers, false));
        assert!(!scope.permits(Ipv4Addr::UNSPECIFIED, &peers, true));
        assert!(!scope.permits(bind, &[], true));
        for peer in [
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::BROADCAST,
            Ipv4Addr::new(224, 0, 0, 1),
        ] {
            assert!(!scope.permits(bind, &[peer], true));
        }
    }

    #[test]
    fn environment_selector_has_explicit_spelling() {
        assert_eq!(
            serde_json::from_str::<NetworkScope>("\"controlled_internet\"").unwrap(),
            NetworkScope::ControlledInternet
        );
        assert!(serde_json::from_str::<NetworkScope>("\"public\"").is_err());
    }
    #[test]
    fn admission_budgets_support_target_transports_without_claiming_player_capacity() {
        let peers = vec![Ipv4Addr::LOCALHOST; 50];
        assert!(admission_budget(256, &peers));
        assert!(admission_budget(65, &peers));
        assert!(admission_budget(1, &[Ipv4Addr::LOCALHOST]));
        assert!(admission_budget(1024, &vec![Ipv4Addr::LOCALHOST; 256]));
        assert!(!admission_budget(0, &peers));
        assert!(!admission_budget(1025, &peers));
        assert!(!admission_budget(usize::MAX, &peers));
        assert!(!admission_budget(256, &[]));
        assert!(!admission_budget(256, &vec![Ipv4Addr::LOCALHOST; 257]));
    }
}
