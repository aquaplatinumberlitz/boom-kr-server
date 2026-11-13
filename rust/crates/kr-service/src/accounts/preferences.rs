//! Account-owned preference, not entitlement, character stats or KR wire.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CharacterPreference {
    pub player_id: i64,
    pub character_key: u16,
    pub revision: i64,
}
fn read(db: &Connection, player: i64) -> Result<CharacterPreference> {
    Ok(db
        .query_row(
            "SELECT character_key,revision FROM player_preferences WHERE player_id=?1",
            [player],
            |row| {
                Ok(CharacterPreference {
                    player_id: player,
                    character_key: row.get(0)?,
                    revision: row.get(1)?,
                })
            },
        )
        .optional()?
        .unwrap_or(CharacterPreference {
            player_id: player,
            character_key: 0,
            revision: 0,
        }))
}
impl Store {
    pub fn character_preference(
        &mut self,
        token: &Token,
        player: i64,
        now: i64,
    ) -> Result<CharacterPreference> {
        let digest = token.digest();
        self.at(now, |tx| {
            Self::session_account(tx, &digest, player, now)?;
            read(tx, player)
        })
    }
    /// The serving choice module decides which character keys are supported.
    /// Persisting a number grants no inventory, stats or client entitlement.
    /// Revision fences delayed writes; retrying the same transition is idempotent.
    pub fn select_character(
        &mut self,
        token: &Token,
        player: i64,
        character: u16,
        expected_revision: i64,
        now: i64,
    ) -> Result<CharacterPreference> {
        if character > 8191 || expected_revision < 0 {
            return Err(Error::InvalidInput);
        }
        let digest = token.digest();
        self.at(now,|tx| {
            Self::session_account(tx,&digest,player,now)?;
            let current=read(tx,player)?;
            let next=expected_revision.checked_add(1).ok_or(Error::Exhausted)?;
            if current.revision==next && current.character_key==character { return Ok(current); }
            if current.revision!=expected_revision { return Err(Error::Conflict); }
            tx.execute("INSERT INTO player_preferences(player_id,character_key,revision) VALUES(?1,?2,?3) ON CONFLICT(player_id) DO UPDATE SET character_key=excluded.character_key,revision=excluded.revision",
                params![player,character,next])?;
            Ok(CharacterPreference{player_id:player,character_key:character,revision:next})
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::fresh;
    fn admin() -> Admin<'static> {
        Admin {
            actor: "M6-pref-test",
            reason: "session preference fences",
        }
    }
    #[test]
    fn selected_character_persists_without_identity_version_change_and_stale_write_cannot_undo_new_choice()
     {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, player) = fresh();
        let grant = store
            .login("alice", b"test-only-password-one", 60, 101)
            .unwrap();
        let initial = store
            .character_preference(&grant.token, player.player_id, 101)
            .unwrap();
        assert_eq!((initial.character_key, initial.revision), (0, 0));
        let chosen = store
            .select_character(&grant.token, player.player_id, 1, 0, 102)
            .unwrap();
        assert_eq!(
            store
                .select_character(&grant.token, player.player_id, 1, 0, 103)
                .unwrap(),
            chosen
        );
        let next = store
            .select_character(&grant.token, player.player_id, 0, 1, 104)
            .unwrap();
        assert_eq!(
            store.select_character(&grant.token, player.player_id, 1, 0, 105),
            Err(Error::Conflict)
        );
        assert_eq!(
            store
                .authorize(&grant.token, player.player_id, 105)
                .unwrap(),
            player
        );
        drop(store);
        let mut store = temp.store();
        assert_eq!(
            store
                .character_preference(&grant.token, player.player_id, 105)
                .unwrap(),
            next
        );
        store.begin_instance(&admin(), 106).unwrap();
        assert_eq!(
            store.select_character(&grant.token, player.player_id, 1, 2, 107),
            Err(Error::InvalidSession)
        );
        let fresh = store
            .login("alice", b"test-only-password-one", 60, 108)
            .unwrap();
        assert_eq!(
            store
                .character_preference(&fresh.token, player.player_id, 108)
                .unwrap(),
            next
        );
    }
    #[test]
    fn foreign_expired_revoked_and_busy_preferences_cannot_mutate_or_publish_success() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, a) = fresh();
        let b = store
            .create_account(&admin(), "bob", "Bob", b"another-test-password", 101)
            .unwrap();
        let grant = store
            .login("alice", b"test-only-password-one", 10, 101)
            .unwrap();
        assert_eq!(
            store.select_character(&grant.token, b.player_id, 1, 0, 102),
            Err(Error::InvalidSession)
        );
        let mut writer = temp.store();
        let lock = writer
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        assert_eq!(
            store.select_character(&grant.token, a.player_id, 1, 0, 102),
            Err(Error::Busy)
        );
        lock.rollback().unwrap();
        assert_eq!(
            store
                .character_preference(&grant.token, a.player_id, 102)
                .unwrap()
                .revision,
            0
        );
        assert_eq!(
            store.select_character(&grant.token, a.player_id, 1, 0, 111),
            Err(Error::InvalidSession)
        );
        let grant = store
            .login("alice", b"test-only-password-one", 60, 112)
            .unwrap();
        store.revoke(&admin(), a.account_id, 113).unwrap();
        assert_eq!(
            store.select_character(&grant.token, a.player_id, 1, 0, 114),
            Err(Error::InvalidSession)
        );
        assert_eq!(
            store
                .db
                .query_row("SELECT count(*) FROM player_preferences", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
