use alloc::vec::Vec;

#[derive(Debug)]
pub struct Response(Vec<u8>);

impl Response {
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut bytes = value.to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }

    pub fn from_int(value: i64) -> Self {
        let mut bytes = value.to_be_bytes().to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }

    pub fn from_list() -> Self {
        // let mut bytes = value.as_bytes().to_vec();
        let mut bytes = Vec::new();
        bytes.push(b'\n');

        Self(bytes)
    }

    pub fn from_usize(value: usize) -> Self {
        let mut bytes = value.to_be_bytes().to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }

    pub fn from_string(value: &str) -> Self {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(b'\n');

        Self(bytes)
    }
}

impl<T: Into<Vec<u8>>> From<T> for Response {
    fn from(v: T) -> Self {
        let mut vec: Vec<u8> = v.into();

        if !vec.ends_with(&[b'\n']) {
            vec.push(b'\n');
        }

        Self(vec)
    }
}
