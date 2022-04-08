use codec::Encode;
use desub_current::decoder;
use desub_current::value::{Composite, Primitive, Value};
use desub_current::Metadata;
use sp_core::crypto::AccountId32;
use sp_core::ByteArray;

pub fn decode_account_id(un: Vec<Value>) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let mut account_id: Vec<u8> = vec![];
    for c in &un {
        match c {
            Value::Primitive(Primitive::U64(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::U32(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::U16(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::U8(inner)) => {
                account_id.push(*inner as u8);
            }

            Value::Primitive(Primitive::I64(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::I32(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::I16(inner)) => {
                account_id.push(*inner as u8);
            }
            Value::Primitive(Primitive::I8(inner)) => {
                account_id.push(*inner as u8);
            }

            _ => {
                continue;
            }
        }
    }

    // TODO: use map_err or other
    match AccountId32::from_slice(&account_id) {
        Ok(account) => Ok(account),
        Err(()) => Err(Box::new(crate::DecodeAccountIdFailed)),
    }
}

pub fn system_account_key(account_id: AccountId32) -> Vec<u8> {
    let mut key = sp_core::twox_128("System".as_bytes()).to_vec();
    key.extend(sp_core::twox_128("Account".as_bytes()).iter());
    let addr_encode = account_id.encode();
    key.extend(sp_core::blake2_128(&addr_encode));
    key.extend(&addr_encode); // blake2_128_concat

    key
}

pub fn user_credit_key(account_id: AccountId32) -> Vec<u8> {
    let mut key = sp_core::twox_128("Credit".as_bytes()).to_vec();
    key.extend(sp_core::twox_128("UserCredit".as_bytes()).iter());
    let addr_encode = account_id.encode();
    key.extend(sp_core::blake2_128(&addr_encode));
    key.extend(&addr_encode); // blake2_128_concat

    key
}

pub fn event_key() -> Vec<u8> {
    let mut key = sp_core::twox_128("System".as_bytes()).to_vec();
    key.extend(sp_core::twox_128("Events".as_bytes()).iter());

    key
}

pub fn decode_storage(storage_key: &str, storage_val: &str, meta: Metadata) -> Value {
    let storage = decoder::decode_storage(&meta);
    let key_bytes = hex::decode(storage_key).unwrap();
    let entry = storage
        .decode_key(&meta, &mut key_bytes.as_slice())
        .expect("can decode storage");
    decoder::decode_value_by_id(
        &meta,
        &entry.ty,
        &mut hex::decode(&storage_val).unwrap().as_slice(),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use sp_core::crypto::Ss58Codec;

    use super::*;

    #[test]
    fn test_system_account_key() {
        let test_addr =
            AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
                .unwrap();
        let key = system_account_key(test_addr);

        assert_eq!(hex::encode(key), "26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da93594ef778a4003043f6d977057644d65a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e");
    }

    #[test]
    fn test_user_credit_key() {
        let test_addr =
            AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
                .unwrap();
        let key = user_credit_key(test_addr);

        assert_eq!(hex::encode(key), "83e0731810368fb22559f084ed61d427f7eb0b356c4455f32f2dab8a7aa408d83594ef778a4003043f6d977057644d65a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e");
    }

    #[test]
    fn test_decode_storage() {
        let storage_key = "83e0731810368fb22559f084ed61d427f7eb0b356c4455f32f2dab8a7aa408d83594ef778a4003043f6d977057644d65a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e";
        let storage_val = "01006400000000000000010000000000010e010000";
        let meta = crate::deeper_metadata();

        let val = decode_storage(storage_key, storage_val, meta);
        match val {
            Value::Composite(Composite::Named(data)) => {
                let credit_val = data[1].1.clone();

                assert_eq!(credit_val, Value::Primitive(Primitive::U64(100)));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_event_key() {
        let key = event_key();

        assert_eq!(
            "26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7",
            hex::encode(key)
        );
    }
}
