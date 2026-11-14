use argon2::password_hash::{PasswordHash as ArgonHash, SaltString};
use argon2::{Argon2, PasswordHasher as Argon2Hasher, PasswordVerifier};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use todooshka_app::{PasswordHasher, RepositoryError, SessionTokens};
use todooshka_domain::PasswordHash;

/// Argon2id with a salt per password: the hash is self-contained for verification.
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| RepositoryError::Unavailable(format!("argon2: {error}")))?;

        PasswordHash::parse(hashed.to_string().as_str())
            .map_err(|error| RepositoryError::Unavailable(format!("argon2: {error}")))
    }

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool {
        let Ok(parsed) = ArgonHash::new(hash.as_str()) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

/// A session token is 32 random bytes in hex; only its SHA-256 reaches the DB.
pub struct RandomSessionTokens;

impl SessionTokens for RandomSessionTokens {
    fn issue(&self) -> (String, String) {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let hash = self.hash(&token);
        (token, hash)
    }

    fn hash(&self, token: &str) -> String {
        hex::encode(Sha256::digest(token.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "correct-horse-battery";

    #[test]
    fn hash_and_verify_roundtrip() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash(PASSWORD).expect("хеш считается");

        assert!(hasher.verify(PASSWORD, &hash));
        assert!(!hasher.verify("another-password", &hash));
        assert_ne!(hash.as_str(), PASSWORD);
        assert!(hash.as_str().starts_with("$argon2"));
    }

    #[test]
    fn same_password_gets_different_salts() {
        let hasher = Argon2PasswordHasher;
        let first = hasher.hash(PASSWORD).expect("хеш");
        let second = hasher.hash(PASSWORD).expect("хеш");

        assert_ne!(first, second, "соль должна быть случайной");
    }

    #[test]
    fn garbage_hash_never_verifies() {
        let hasher = Argon2PasswordHasher;
        let hash = PasswordHash::parse("not-an-argon2-hash").expect("непустая строка");

        assert!(!hasher.verify(PASSWORD, &hash));
    }

    #[test]
    fn tokens_are_random_and_stored_as_hashes() {
        let tokens = RandomSessionTokens;
        let (first, first_hash) = tokens.issue();
        let (second, second_hash) = tokens.issue();

        assert_ne!(first, second, "токены не должны повторяться");
        assert_eq!(first_hash, tokens.hash(&first));
        assert_ne!(first_hash, first);
        assert_eq!(first_hash.len(), 64, "SHA-256 в hex");
        assert_ne!(first_hash, second_hash);
    }
}
