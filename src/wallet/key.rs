use secp256k1::{SecretKey, PublicKey};
use sha2::{Sha256, Digest};
use rand::thread_rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Address([u8; 32]);

pub struct KeyPair {
    pub secret: SecretKey,
    pub public: PublicKey,
    pub address: Address,
}

impl KeyPair {
    pub fn generate() -> Self {
        let secp = secp256k1::Secp256k1::new();
        let mut rng = thread_rng();
        let (sk, pk) = secp.generate_keypair(&mut rng);
        
        let mut hasher = Sha256::new();
        hasher.update(pk.serialize());
        let hash = hasher.finalize();
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&hash);
        
        Self {
            secret: sk,
            public: pk,
            address: Address(addr),
        }
    }
    
    pub fn address_string(&self) -> String {
        hex::encode(self.address.0)
    }
}

impl Address {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn to_string(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}
