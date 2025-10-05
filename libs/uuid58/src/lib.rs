use bs58;
use uuid::Uuid;

pub fn new() -> String {
    let uuid = Uuid::new_v4();
    bs58::encode(uuid.as_bytes()).into_string()
}
