use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::OnceCell;

pub fn serialize<S, T>(value: &OnceCell<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    value.get().serialize(serializer)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<OnceCell<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(|v| {
        let cell = OnceCell::new();
        //TODO don't eat
        cell.set(v).ok();
        cell
    })
}
