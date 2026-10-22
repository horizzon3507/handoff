use subtle::ConstantTimeEq;
use uuid::Uuid;

#[derive(Clone)]
pub struct RoomToken(String);

impl RoomToken {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, candidate: &str) -> bool {
        self.0.len() == candidate.len()
            && bool::from(self.0.as_bytes().ct_eq(candidate.as_bytes()))
    }
}
