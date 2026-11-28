use kr_wire::{
    CodecError, FRAME_OVERHEAD, MAX_FRAME_LEN,
    kr00967::{
        self, ChatServerConnectReply, F5_C2S_COUPLE_670, F5_C2S_GUILD_INVITE_AUTO_REFUSE,
        F5_S2C_COUPLE_257, F5_S2C_COUPLE_273, F5_S2C_COUPLE_284, F5_S2C_COUPLE_640,
        F5_S2C_COUPLE_674, F5_S2C_GUILD_467, F5_S2C_GUILD_INVITE, F5_S2C_REFUSE_GUILD_INVITE,
        F5_SOCIAL_ANCHORS, F5AnchorSite, F5ConfirmedBody, F5Direction, F5SocialAnchor,
        F5SocialGroup, F5WireContract, GuildInvite, GuildInviteAutoRefuse, f5_social_opcode_anchor,
    },
};
use std::{fmt::Debug, io, num::NonZeroU8};

// Synthetic bodies from the IDA field order and KR primitive masks, not captures.
const INVITE_BODY: [u8; 19] = [
    0xa5, 0x68, 0x1b, // "A"
    0xa5, 0x6b, 0x74, 0xae, // "한" (C7 D1)
    0x97, 0xc8, 0x50, 0x91, // first u32
    0x67, 0xc8, 0x50, 0x91, // second u32
    0xa5, 0x6b, 0x84, 0x33, // "Z!"
];
const AUTO_REFUSE_BODY: [u8; 8] = [0x5a, 0xa5, 0x6b, 0x84, 0x33, 0xa5, 0x68, 0x1b];
const FAILED_CONNECT_BODY: [u8; 1] = [0x5a];
const ENDPOINT_BODY: [u8; 12] = [
    0x58, 0x5d, 0x97, 0xc8, 0x50, 0x91, 0x94, 0xca, 0x53, 0xea, 0xae, 0xd1,
];

#[test]
fn guild_bodies_round_trip_with_static_field_order() {
    let invite = GuildInvite {
        first: "A".into(),
        second: "한".into(),
        first_number: 0x0102_0304,
        second_number: 0xf102_0304,
        third: "Z!".into(),
    };
    assert_eq!(invite.encode().unwrap(), INVITE_BODY);
    assert_eq!(GuildInvite::decode(&INVITE_BODY).unwrap(), invite);

    let refusal = GuildInviteAutoRefuse {
        invite_third: invite.third,
        invite_first: invite.first,
    };
    assert_eq!(refusal.encode().unwrap(), AUTO_REFUSE_BODY);
    assert_eq!(
        GuildInviteAutoRefuse::decode(&AUTO_REFUSE_BODY).unwrap(),
        refusal
    );

    let mut other_branch = AUTO_REFUSE_BODY;
    other_branch[0] = 0x5b; // Nonzero discriminator is outside the automatic branch.
    assert_error_kind(
        GuildInviteAutoRefuse::decode(&other_branch),
        io::ErrorKind::InvalidData,
    );
}

#[test]
fn chat_reply_round_trips_both_status_branches() {
    for (body, reply) in [
        (&FAILED_CONNECT_BODY[..], ChatServerConnectReply::Failed),
        (
            &ENDPOINT_BODY[..],
            ChatServerConnectReply::Endpoint {
                ok: NonZeroU8::new(2).unwrap(),
                second_byte: 7,
                first_number: 0x0102_0304,
                ip: 0x0200_007f,
                port: 3000,
            },
        ),
    ] {
        assert_eq!(reply.encode().unwrap(), body);
        assert_eq!(ChatServerConnectReply::decode(body).unwrap(), reply);
    }
}

fn assert_error_kind<T: Debug>(result: io::Result<T>, kind: io::ErrorKind) {
    assert_eq!(result.unwrap_err().kind(), kind);
}

fn assert_complete_body<T: Debug>(body: &[u8], decode: impl Fn(&[u8]) -> io::Result<T>) {
    for end in 0..body.len() {
        assert_eq!(
            decode(&body[..end]).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof,
            "prefix of {end} bytes"
        );
    }
    let mut trailing = body.to_vec();
    trailing.push(0);
    assert_error_kind(decode(&trailing), io::ErrorKind::InvalidData);
}

#[test]
fn all_confirmed_bodies_reject_truncation_and_trailing_bytes() {
    assert_complete_body(&INVITE_BODY, GuildInvite::decode);
    assert_complete_body(&AUTO_REFUSE_BODY, GuildInviteAutoRefuse::decode);
    assert_complete_body(&FAILED_CONNECT_BODY, ChatServerConnectReply::decode);
    assert_complete_body(&ENDPOINT_BODY, ChatServerConnectReply::decode);
}

#[test]
fn guild_codecs_accept_exact_frame_budget_and_reject_one_more_byte() {
    const MAX_BODY: usize = MAX_FRAME_LEN - FRAME_OVERHEAD;
    // Three string lengths, two numbers, and the two CP949 bytes for "한".
    const INVITE_PREFIX_BYTES: usize = 3 * 2 + 2 * 4 + 2;
    let mut invite = GuildInvite {
        first: "한".into(),
        second: String::new(),
        first_number: 0,
        second_number: u32::MAX,
        third: "a".repeat(MAX_BODY - INVITE_PREFIX_BYTES),
    };
    let mut refusal = GuildInviteAutoRefuse {
        invite_third: String::new(),
        invite_first: "a".repeat(MAX_BODY - 5),
    };
    let invite_body = invite.encode().unwrap();
    let refusal_body = refusal.encode().unwrap();
    assert_eq!(GuildInvite::decode(&invite_body).unwrap(), invite);
    assert_eq!(
        GuildInviteAutoRefuse::decode(&refusal_body).unwrap(),
        refusal
    );

    for (opcode, mut body) in [
        (GuildInvite::OPCODE, invite_body),
        (GuildInviteAutoRefuse::OPCODE, refusal_body),
    ] {
        assert_eq!(body.len(), MAX_BODY);
        let frame = kr00967::encode(opcode, &body, 0).unwrap();
        assert_eq!(frame.len(), MAX_FRAME_LEN);
        body.push(0);
        assert_eq!(
            kr00967::encode(opcode, &body, 0),
            Err(CodecError::FrameTooLarge)
        );
    }

    invite.third.push('a');
    refusal.invite_first.push('a');
    assert_error_kind(invite.encode(), io::ErrorKind::InvalidData);
    assert_error_kind(refusal.encode(), io::ErrorKind::InvalidData);

    // With no size guard these Guild bodies would instead fail on a short string.
    let mut oversized = vec![0; MAX_BODY + 1];
    oversized[..2].copy_from_slice(&[0x5a, 0x96]);
    assert_error_kind(GuildInvite::decode(&oversized), io::ErrorKind::InvalidData);
    oversized[..3].copy_from_slice(&[0x5a, 0x5a, 0x96]);
    assert_error_kind(
        GuildInviteAutoRefuse::decode(&oversized),
        io::ErrorKind::InvalidData,
    );
    assert_error_kind(
        ChatServerConnectReply::decode(&oversized),
        io::ErrorKind::InvalidData,
    );
}

#[test]
fn guild_codecs_reject_invalid_cp949_and_lossy_input() {
    let mut malformed_invite = INVITE_BODY;
    malformed_invite[2] = 0xa5; // Native 0xff is not a CP949 character.
    assert_error_kind(
        GuildInvite::decode(&malformed_invite),
        io::ErrorKind::InvalidData,
    );

    let mut malformed_refusal = AUTO_REFUSE_BODY;
    malformed_refusal[7] = 0xdb; // Native 0x81 is an incomplete CP949 lead byte.
    assert_error_kind(
        GuildInviteAutoRefuse::decode(&malformed_refusal),
        io::ErrorKind::InvalidData,
    );

    let mut invite = GuildInvite::decode(&INVITE_BODY).unwrap();
    let mut refusal = GuildInviteAutoRefuse::decode(&AUTO_REFUSE_BODY).unwrap();
    invite.second = "한🦀".into();
    refusal.invite_first = "🦀".into();
    assert_error_kind(invite.encode(), io::ErrorKind::InvalidData);
    assert_error_kind(refusal.encode(), io::ErrorKind::InvalidData);
}

#[test]
fn declared_string_length_must_fit_in_remaining_body() {
    let mut invite = INVITE_BODY;
    invite[..2].copy_from_slice(&[0x5a, 0x96]); // Native u16::MAX.
    assert_error_kind(GuildInvite::decode(&invite), io::ErrorKind::UnexpectedEof);

    let mut refusal = AUTO_REFUSE_BODY;
    refusal[1..3].copy_from_slice(&[0x5a, 0x96]);
    assert_error_kind(
        GuildInviteAutoRefuse::decode(&refusal),
        io::ErrorKind::UnexpectedEof,
    );
}

fn assert_opcode_anchor(expected: F5SocialAnchor, opcode: u32) {
    assert_eq!(expected.opcode, Some(opcode));
    assert_eq!(
        f5_social_opcode_anchor(expected.direction, opcode),
        Some(&expected)
    );
    let reverse = match expected.direction {
        F5Direction::ServerToClient => F5Direction::ClientToServer,
        F5Direction::ClientToServer => F5Direction::ServerToClient,
    };
    assert_eq!(f5_social_opcode_anchor(reverse, opcode), None);
}

#[test]
fn guild_anchors_preserve_ida_sites_and_only_two_confirmed_bodies() {
    use F5AnchorSite::{CStageHandler, RequestBuilderCallers};
    use F5Direction::{ClientToServer, ServerToClient};
    use F5WireContract::{KrStaticConfirmed, UnmeasuredWireContract};

    for (opcode, expected, direction, name, site, wire_contract) in [
        (
            F5_S2C_GUILD_467,
            467,
            ServerToClient,
            None,
            CStageHandler(0x6D9340),
            UnmeasuredWireContract,
        ),
        (
            F5_S2C_GUILD_INVITE,
            474,
            ServerToClient,
            Some("GuildInvite"),
            CStageHandler(0xD769D0),
            KrStaticConfirmed(F5ConfirmedBody::GuildInvite),
        ),
        (
            F5_S2C_REFUSE_GUILD_INVITE,
            478,
            ServerToClient,
            Some("RefuseGuildInvite"),
            CStageHandler(0xD76DA0),
            UnmeasuredWireContract,
        ),
        (
            F5_C2S_GUILD_INVITE_AUTO_REFUSE,
            633,
            ClientToServer,
            Some("GuildInviteAutoRefuse"),
            RequestBuilderCallers(&[0xD769D0]),
            KrStaticConfirmed(F5ConfirmedBody::GuildInviteAutoRefuse),
        ),
    ] {
        assert_opcode_anchor(
            F5SocialAnchor {
                group: F5SocialGroup::Guild,
                direction,
                opcode: Some(opcode),
                name,
                site,
                wire_contract,
            },
            expected,
        );
    }
    assert_eq!(GuildInvite::OPCODE, 474);
    assert_eq!(GuildInviteAutoRefuse::OPCODE, 633);
}

#[test]
fn couple_anchors_have_no_wire_contract() {
    for (opcode, expected, handler_va) in [
        (F5_S2C_COUPLE_257, 257, 0x128D3C0),
        (F5_S2C_COUPLE_273, 273, 0xD87250),
        (F5_S2C_COUPLE_284, 284, 0x128D120),
        (F5_S2C_COUPLE_640, 640, 0xD88830),
        (F5_S2C_COUPLE_674, 674, 0xD7FA60),
    ] {
        assert_opcode_anchor(
            F5SocialAnchor {
                group: F5SocialGroup::Couple,
                direction: F5Direction::ServerToClient,
                opcode: Some(opcode),
                name: None,
                site: F5AnchorSite::CStageHandler(handler_va),
                wire_contract: F5WireContract::UnmeasuredWireContract,
            },
            expected,
        );
    }
    assert_opcode_anchor(
        F5SocialAnchor {
            group: F5SocialGroup::Couple,
            direction: F5Direction::ClientToServer,
            opcode: Some(F5_C2S_COUPLE_670),
            name: None,
            site: F5AnchorSite::RequestBuilderCallers(&[0x583530, 0x583E80]),
            wire_contract: F5WireContract::UnmeasuredWireContract,
        },
        670,
    );
}

#[test]
fn chat_connect_anchor_confirms_only_the_body() {
    assert_eq!(F5_SOCIAL_ANCHORS.len(), 11);
    assert!(F5_SOCIAL_ANCHORS.contains(&F5SocialAnchor {
        group: F5SocialGroup::Chat,
        direction: F5Direction::ServerToClient,
        opcode: None,
        name: Some("CChattingStage::OnResConnectToSvr"),
        site: F5AnchorSite::ChatStageHandler {
            handler_va: 0xD58760,
            connect_callee_va: 0x13AD4D0,
        },
        wire_contract: F5WireContract::KrStaticConfirmed(F5ConfirmedBody::ChatServerConnectReply),
    }));
    assert_eq!(
        f5_social_opcode_anchor(F5Direction::ServerToClient, 0),
        None
    );
}
