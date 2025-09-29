use ring::{
    aead::{self, AES_256_GCM, Aad, LessSafeKey, Nonce},
    hkdf::{HKDF_SHA256, Salt},
};
use x25519_dalek::SharedSecret;

pub struct Encryption {}

impl Encryption {
    pub fn derive_key(shared: &SharedSecret) -> aead::LessSafeKey {
        let salt = Salt::new(HKDF_SHA256, b"SFT file transfer v1");
        let prk = salt.extract(shared.as_bytes());
        let okm = prk
            .expand(&[b"session key"], &AES_256_GCM)
            .expect("HKDF expand failed");

        let mut key_byte = [0u8; 32];
        okm.fill(&mut key_byte).expect("HKDF fill failed");
        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key_byte).expect("invalid AEAD key");
        aead::LessSafeKey::new(unbound)
    }

    pub fn encrypt(key: &LessSafeKey, plaintext: &mut Vec<u8>, nonce_counter: u64) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&nonce_counter.to_be_bytes());
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        key.seal_in_place_separate_tag(nonce, Aad::empty(), plaintext)
            .expect("encryption failed")
            .as_ref()
            .to_vec()
    }

    pub fn decrypt(key: LessSafeKey, ciphertext: &mut Vec<u8>, nonce_counter: u64) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&nonce_counter.to_be_bytes());
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        key.open_in_place(nonce, Aad::empty(), ciphertext)
            .expect("decryption failed")
            .to_vec()
    }
}
