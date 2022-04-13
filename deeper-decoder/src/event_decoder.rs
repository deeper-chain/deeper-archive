use desub_current::value::{Composite, Value};
use desub_current::Metadata;

// TODO: use jsonb to store event detail may cause performance issue, in the future
// we may need to come up with a new way.
// the old style is to match events we care, but that's too costy.
pub fn decode_event(storage_key: &str, storage_val: &str, meta: Metadata) -> Vec<Value> {
    let event_value = crate::common::decode_storage(storage_key, storage_val, meta);
    match event_value {
        Value::Composite(Composite::Unnamed(events)) => events,
        _ => vec![],
    }
}
