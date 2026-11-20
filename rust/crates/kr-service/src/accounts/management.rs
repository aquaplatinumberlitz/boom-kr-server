use super::*;
use crate::account_serving::Secret;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Management {
    Register {
        display_name: String,
        password: Secret,
    },
    Change {
        current: Secret,
        password: Secret,
    },
    Delete {
        current: Secret,
    },
    Recover {
        code: Secret,
        password: Secret,
    },
}

fn native_password(password: &Secret) -> Result<&[u8]> {
    let bytes = password.0.as_bytes();
    password_bounds(bytes)?;
    if !bytes.iter().all(|byte| (b' '..=b'~').contains(byte)) {
        return Err(Error::InvalidInput);
    }
    Ok(bytes)
}

impl Store {
    pub(super) fn new_recovery_code(&self) -> Result<Zeroizing<String>> {
        let random = Zeroizing::new(self.random::<32>()?);
        Ok(Zeroizing::new(hex(random.as_ref())))
    }
}
impl Store {
    pub(crate) fn manage(
        &mut self,
        login: &str,
        operation: Management,
    ) -> Result<Option<Zeroizing<String>>> {
        let login = login_name(login)?;
        let actor = Admin {
            actor: "player-launcher",
            reason: "self-service",
        };
        let request = hex(&self.random::<32>()?);
        if let Management::Register {
            display_name: name,
            password,
        } = operation
        {
            display_name(&name)?;
            let new = password_hash(native_password(&password)?, &self.random::<16>()?)?;
            let code = self.new_recovery_code()?;
            let digest = Sha256::digest(code.as_bytes());
            self.at_clock(&mut utc, |tx, now| {
                let player = Self::insert_account(tx, &login, &name, &new, &actor, &request, now)?;
                tx.execute(
                    "INSERT INTO account_recovery VALUES(?1,?2)",
                    params![player.account_id, &digest[..]],
                )?;
                Ok(())
            })?;
            return Ok(Some(code));
        }

        let row: Option<(i64, String, i64, bool)> = self
            .db
            .query_row(
                "SELECT id,password_phc,credential_version,disabled FROM accounts WHERE login=?1
             AND NOT EXISTS(SELECT 1 FROM guest_accounts WHERE account_id=accounts.id)",
                [&login],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;

        let verify_current = |current: &Secret| {
            if verify(
                native_password(current)?,
                row.as_ref().map(|r| r.1.as_str()),
            )? {
                Ok(())
            } else {
                Err(Error::InvalidCredentials)
            }
        };
        let (op, new, recovery) = match operation {
            Management::Register { .. } => unreachable!(),
            Management::Change { current, password } => {
                verify_current(&current)?;
                (
                    "change_password",
                    password_hash(native_password(&password)?, &self.random::<16>()?)?,
                    None,
                )
            }
            Management::Delete { current } => {
                verify_current(&current)?;
                let random = Zeroizing::new(hex(Zeroizing::new(self.random::<32>()?).as_ref()));
                (
                    "delete_account",
                    password_hash(random.as_bytes(), &self.random::<16>()?)?,
                    None,
                )
            }
            Management::Recover { code, password } => {
                // 256-bit random code with exact canonical encoding. A conditional
                // DELETE in the mutation transaction provides once-only use.
                let code = Token::parse(&code.0).map_err(|_| Error::InvalidCredentials)?;
                (
                    "recover_account",
                    password_hash(native_password(&password)?, &self.random::<16>()?)?,
                    Some(code.digest()),
                )
            }
        };
        let Some((account, _, version, false)) = row else {
            return Err(Error::InvalidCredentials);
        };
        let replacement = if op == "delete_account" {
            None
        } else {
            Some(self.new_recovery_code()?)
        };
        let issued = self.at_clock(&mut utc, |tx, now| {
            let issued = if let Some(digest) = recovery {
                let new_digest = Sha256::digest(
                    replacement
                        .as_ref()
                        .expect("recovery can rotate")
                        .as_bytes(),
                );
                if tx.execute(
                    "UPDATE account_recovery SET digest=?1 WHERE account_id=?2 AND digest=?3",
                    params![&new_digest[..], account, digest],
                )? != 1
                {
                    return Err(Error::InvalidCredentials);
                }
                true
            } else if let Some(code) = &replacement {
                let digest = Sha256::digest(code.as_bytes());
                tx.execute(
                    "INSERT INTO account_recovery VALUES(?1,?2) ON CONFLICT(account_id) DO NOTHING",
                    params![account, &digest[..]],
                )? == 1
            } else {
                false
            };
            if tx.execute(
                "UPDATE accounts SET password_phc=?1, credential_version=credential_version+1
                 WHERE id=?2 AND credential_version=?3 AND disabled=0",
                params![new, account, version],
            )? != 1
            {
                return Err(Error::InvalidCredentials);
            }
            tx.execute(
                "UPDATE sessions SET revoked_at=?1 WHERE account_id=?2 AND revoked_at IS NULL",
                params![now, account],
            )?;
            if op == "delete_account" {
                Self::tombstone(tx, account)?;
            }
            Self::audit(tx, &request, &actor, op, Some(account), now)?;
            Ok(issued)
        })?;
        Ok(replacement.filter(|_| issued))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::Temp;
    use super::*;

    fn secret(text: &str) -> Secret {
        Secret(Zeroizing::new(text.into()))
    }

    #[test]
    fn recovery_is_single_use_and_all_credential_changes_revoke_exact_account() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let temp = Temp::new();
        let mut store = temp.store();
        let admin = Admin {
            actor: "test",
            reason: "launcher management lifecycle",
        };
        store.begin_instance(&admin, utc().unwrap()).unwrap();
        let first = "first-test-only-password";
        let second = "second-test-only-password";
        let third = "third-test-only-password";
        let code = store
            .manage(
                "Tester",
                Management::Register {
                    display_name: "Tester".into(),
                    password: secret(first),
                },
            )
            .unwrap()
            .unwrap();
        let session = store.login_current("tester", first.as_bytes(), 60).unwrap();
        let player = session.player.player_id;
        let saved: Vec<u8> = store
            .db
            .query_row("SELECT digest FROM account_recovery", [], |r| r.get(0))
            .unwrap();
        assert_eq!(saved, Sha256::digest(code.as_bytes()).to_vec());
        assert_eq!(
            store.manage(
                "Tester",
                Management::Change {
                    current: secret(second),
                    password: secret(third),
                }
            ),
            Err(Error::InvalidCredentials)
        );
        store
            .manage(
                "tester",
                Management::Change {
                    current: secret(first),
                    password: secret(second),
                },
            )
            .unwrap();
        assert!(
            store
                .authorize(&session.token, player, utc().unwrap())
                .is_err()
        );
        let session = store
            .login_current("tester", second.as_bytes(), 60)
            .unwrap();
        let next = store
            .manage(
                "TESTER",
                Management::Recover {
                    code: secret(&code),
                    password: secret(third),
                },
            )
            .unwrap()
            .unwrap();
        assert_ne!(*code, *next);
        assert!(
            store
                .authorize(&session.token, player, utc().unwrap())
                .is_err()
        );
        assert_eq!(
            store.manage(
                "tester",
                Management::Recover {
                    code: secret(&code),
                    password: secret(second),
                }
            ),
            Err(Error::InvalidCredentials)
        );
        assert!(
            store
                .manage(
                    "tester",
                    Management::Recover {
                        code: secret(&next),
                        password: secret(first),
                    }
                )
                .unwrap()
                .is_some()
        );
        let session = store.login_current("tester", first.as_bytes(), 60).unwrap();
        store
            .manage(
                "tester",
                Management::Delete {
                    current: secret(first),
                },
            )
            .unwrap();
        assert!(
            store
                .authorize(&session.token, player, utc().unwrap())
                .is_err()
        );
        assert!(store.lookup("tester").is_err());
        store
            .manage(
                "tester",
                Management::Register {
                    display_name: "Tester".into(),
                    password: secret(first),
                },
            )
            .unwrap();
        assert_ne!(store.lookup("tester").unwrap().player_id, player);
    }
    #[test]
    fn first_password_change_enrolls_an_existing_account_without_a_code() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, player) = super::super::tests::fresh();
        let login = "Alice";
        let existing = "test-only-password-one";
        let updated = "test-only-changed-password";
        let code = store
            .manage(
                login,
                Management::Change {
                    current: secret(existing),
                    password: secret(updated),
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(store.lookup(login).unwrap().player_id, player.player_id);
        drop(store);
        let mut store = temp.store();
        assert!(
            store
                .manage(
                    login,
                    Management::Change {
                        current: secret(updated),
                        password: secret(existing),
                    }
                )
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .manage(
                    login,
                    Management::Recover {
                        code: secret(&code),
                        password: secret(updated),
                    }
                )
                .unwrap()
                .is_some()
        );
    }
}
