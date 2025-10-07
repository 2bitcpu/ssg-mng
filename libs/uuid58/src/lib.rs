use bs58;
use uuid::Uuid;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub fn new_string() -> String {
    bs58::encode(Uuid::new_v4().as_bytes()).into_string()
}

pub fn encode(uuid_value: Uuid) -> String {
    bs58::encode(uuid_value.as_bytes()).into_string()
}

pub fn encode_from_str(uuid_str: &str) -> Result<String, BoxError> {
    let uuid_value = Uuid::parse_str(uuid_str)?;
    Ok(encode(uuid_value))
}

pub fn decode(uuid58_str: &str) -> Result<Uuid, BoxError> {
    let decoded_bytes = bs58::decode(uuid58_str).into_vec()?;
    let uuid_bytes: [u8; 16] = decoded_bytes
        .try_into()
        .map_err(|_| "Decoded string is not a valid UUID length")?;
    Ok(Uuid::from_bytes(uuid_bytes))
}

pub fn decode_to_string(uuid58_str: &str) -> Result<String, BoxError> {
    let uuid_value = decode(uuid58_str)?;
    Ok(uuid_value.to_string())
}
