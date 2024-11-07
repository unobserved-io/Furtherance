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
use sysinfo::System;

#[derive(Debug)]
pub enum EncryptionError {
    KeyDerivation,
    Encryption,
    Decryption,
    Serialization,
    DeviceId,
}

const FURTHERANCE_SALT: &[u8; 32] = &[
    0x8a, 0x24, 0x3b, 0x71, 0x9c, 0xf5, 0xe2, 0x16, 0x4d, 0x8e, 0x57, 0x39, 0x0a, 0xb1, 0xca, 0x84,
    0x6f, 0x92, 0xd3, 0x45, 0x1e, 0x7b, 0xc8, 0xf0, 0x5d, 0x9a, 0x36, 0x82, 0x4c, 0xb5, 0xe7, 0x1d,
];

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

/// Generate encryption key from password
pub fn derive_encryption_key(password: &str) -> Result<[u8; 32], EncryptionError> {
    let argon2 = Argon2::default();

    let salt =
        SaltString::encode_b64(FURTHERANCE_SALT).map_err(|_| EncryptionError::KeyDerivation)?;

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| EncryptionError::KeyDerivation)
        .map(|hash| {
            let mut key = [0u8; 32];
            key.copy_from_slice(&hash.hash.unwrap().as_bytes()[0..32]);
            key
        })
}

/// Generate device-specific key for storing encryption key
fn get_device_key() -> Result<[u8; 32], EncryptionError> {
    let device_id = generate_device_id()?;

    let mut key = [0u8; 32];
    let mut hasher = blake3::Hasher::new();
    hasher.update(device_id.as_bytes());
    key.copy_from_slice(&hasher.finalize().as_bytes()[..32]);

    Ok(key)
}

pub fn encrypt_encryption_key(
    encryption_key: &[u8; 32],
) -> Result<(String, String), EncryptionError> {
    let device_key = get_device_key()?;
    encrypt(encryption_key, &device_key)
}

pub fn decrypt_encryption_key(
    encrypted_key: &str,
    nonce: &str,
) -> Result<[u8; 32], EncryptionError> {
    let device_key = get_device_key()?;
    decrypt(encrypted_key, nonce, &device_key)
}

/// Combine multiple sources for unique device ID
pub fn generate_device_id() -> Result<String, EncryptionError> {
    let machine_id = machine_uid::get().map_err(|_| EncryptionError::DeviceId)?;
    let hostname = System::host_name().unwrap_or_default();
    Ok(format!("{}:{}", machine_id, hostname))
}
