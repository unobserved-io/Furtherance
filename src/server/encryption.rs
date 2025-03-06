// Furtherance - Track your time without being tracked
// Copyright (C) 2025  Ricky Kresslein <rk@unobserved.io>
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
use base64::{
    engine::general_purpose::{STANDARD as BASE64, URL_SAFE_NO_PAD},
    Engine,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug)]
pub enum EncryptionError {
    Encryption,
    Decryption,
    Serialization,
    DeviceId,
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
    encryption_key: &String,
) -> Result<(String, String), EncryptionError> {
    let device_key = get_device_key()?;
    encrypt(encryption_key, &device_key)
}

pub fn decrypt_encryption_key(
    encrypted_key: &str,
    nonce: &str,
) -> Result<[u8; 32], EncryptionError> {
    let device_key = get_device_key()?;
    let key_string: String = decrypt(encrypted_key, nonce, &device_key)?;
    let key_bytes = URL_SAFE_NO_PAD
        .decode(key_string)
        .map_err(|_| EncryptionError::Serialization)?;

    if key_bytes.len() != 32 {
        return Err(EncryptionError::Serialization);
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&key_bytes);

    Ok(result)
}

/// Combine multiple sources for unique device ID
pub fn generate_device_id() -> Result<String, EncryptionError> {
    let machine_id = machine_uid::get().map_err(|_| EncryptionError::DeviceId)?;
    let hostname = System::host_name().unwrap_or_default();
    Ok(format!("{}:{}", machine_id, hostname))
}
