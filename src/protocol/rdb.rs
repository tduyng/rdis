pub struct Rdb {}

impl Rdb {
    pub fn get_empty() -> Vec<u8> {
        let empty_writer = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
        Self::hex_to_bytes(empty_writer)
    }

    pub fn hex_to_bytes(content: &str) -> Vec<u8> {
        let byte_content = hex::decode(content).unwrap();
        let mut result = format!("${}\r\n", byte_content.len()).into_bytes();
        result.extend(byte_content);
        result
    }
}
