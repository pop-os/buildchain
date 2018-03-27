#[repr(packed)]
pub struct Block {
    signature: [u8; 64],
    public_key: [u8; 32],
    previous_signature: [u8; 64],
    counter: [u8; 8],
    timestamp: [u8; 8],
    digest: [u8; 48],
}