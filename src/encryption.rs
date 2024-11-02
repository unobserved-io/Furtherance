// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum EncryptionError {
    KeyDerivation,
    Encryption,
    Decryption,
    Serialization,
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], EncryptionError> {
    let salt_string = SaltString::encode_b64(salt).map_err(|_| EncryptionError::KeyDerivation)?;
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|_| EncryptionError::KeyDerivation)
        .map(|hash| {
            let mut key = [0u8; 32];
            key.copy_from_slice(&hash.hash.unwrap().as_bytes()[0..32]);
            key
        })
}

pub fn encrypt<T: Serialize>(
    data: &T,
    key: &[u8; 32],
) -> Result<(String, String), EncryptionError> {
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let json = serde_json::to_string(data).map_err(|_| EncryptionError::Serialization)?;

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let ciphertext = cipher
        .encrypt(nonce, json.as_bytes())
        .map_err(|_| EncryptionError::Encryption)?;

    Ok((BASE64.encode(ciphertext), BASE64.encode(nonce_bytes)))
}

pub fn decrypt<T: for<'de> Deserialize<'de>>(
    encrypted_data: &str,
    nonce_b64: &str,
    key: &[u8; 32],
) -> Result<T, EncryptionError> {
    let ciphertext = BASE64
        .decode(encrypted_data)
        .map_err(|_| EncryptionError::Decryption)?;
    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|_| EncryptionError::Decryption)?;

    // Ensure nonce is correct size
    if nonce_bytes.len() != 12 {
        return Err(EncryptionError::Decryption);
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| EncryptionError::Decryption)?;

    serde_json::from_slice(&plaintext).map_err(|_| EncryptionError::Serialization)
}
