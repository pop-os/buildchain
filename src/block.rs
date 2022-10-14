use plain::{self, Plain};
use serde::{Deserialize, Serialize};
use sodalite::sign_attached_open;

use crate::store::b32enc;

#[allow(dead_code)]
#[repr(packed)]
pub (crate) struct PackedBlockRequest {
    signature: [u8; 64],
    public_key: [u8; 32],
    previous_signature: [u8; 64],
    counter: [u8; 8],
    timestamp: [u8; 8],
    digest: [u8; 48],
}

#[repr(packed)]
pub (crate) struct PackedBlock {
    signature: [u8; 64],
    public_key: [u8; 32],
    previous_signature: [u8; 64],
    counter: u64,
    timestamp: u64,
    request: PackedBlockRequest,
}

unsafe impl Plain for PackedBlock {}

impl PackedBlock {
    // Convert to a usable struct through verification
    pub (crate) fn verify(&self, key: &[u8]) -> Result<Block, String> {
        if self.public_key != key {
            return Err("public key mismatch".to_string());
        }

        {
            let sm = unsafe { plain::as_bytes(self) };

            let mut m = vec![0; sm.len()];
            match sign_attached_open(&mut m, sm, &self.public_key) {
                Ok(count) => m.truncate(count),
                Err(()) => return Err("signature invalid".to_string()),
            }

            // Check that message matches signed message after skipping the signature
            if m != sm[64..] {
                return Err("message data invalid".to_string());
            }
        }

        Ok(Block {
            signature: b32enc(&self.signature),
            public_key: b32enc(&self.public_key),
            previous_signature: b32enc(&self.previous_signature),
            counter: u64::from_le(self.counter),
            timestamp: u64::from_le(self.timestamp),
            digest: b32enc(&self.request.digest),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub signature: String,
    pub public_key: String,
    pub previous_signature: String,
    pub counter: u64,
    pub timestamp: u64,
    pub digest: String,
}
