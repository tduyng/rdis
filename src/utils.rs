use rand::Rng;
use sha1::{Digest, Sha1};

pub fn current_time_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_millis()
}

pub fn random_sha1_hex() -> String {
    let mut rng = rand::thread_rng();
    let mut sha1 = Sha1::new();
    let random_bytes: [u8; 20] = rng.gen();
    sha1.update(random_bytes);
    let hash_bytes = sha1.finalize();
    hex::encode(hash_bytes)
}
