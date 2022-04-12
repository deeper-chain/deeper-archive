use desub_current::value::{self, Composite, Value};
use desub_current::Metadata;
use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum EventInfo {
    CreditDataAdded(AccountId32, CreditData),
    CreditDataUpdated(AccountId32, CreditData),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CreditData {
    pub credit: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EventRecord {
    pub pallet: String,
    pub name: String,
    pub info: EventInfo,
}

pub fn decode_event(storage_key: &str, storage_val: &str, meta: Metadata) -> Vec<EventRecord> {
    let mut res: Vec<EventRecord> = vec![];

    let event_value = crate::common::decode_storage(storage_key, storage_val, meta);
    match event_value {
        Value::Composite(Composite::Unnamed(events)) => {
            for event in &events {
                match event {
                    Value::Composite(Composite::Named(ev1)) => {
                        if ev1.len() == 3 {
                            match ev1[1].1.clone() {
                                Value::Variant(ev2) => {
                                    let pallet_name = ev2.name.clone();
                                    match ev2.values {
                                        Composite::Unnamed(ev3) => {
                                            if ev3.len() == 1 {
                                                match ev3[0].clone() {
                                                    Value::Variant(ev4) => {
                                                        let event_name = ev4.name.clone();
                                                        // TODO: match other events here
                                                        if &event_name == "CreditDataAdded" {
                                                            let event_data =
                                                                decode_credit_data_added(
                                                                    ev4.values,
                                                                );
                                                            match event_data {
                                                                Ok(event_data_inner) => {
                                                                    let record = EventRecord {
                                                                        pallet: pallet_name,
                                                                        name: event_name,
                                                                        info: event_data_inner,
                                                                    };
                                                                    res.push(record);
                                                                }
                                                                _ => {}
                                                            }
                                                        } else if &event_name == "CreditDataUpdated"
                                                        {
                                                            let event_data =
                                                                decode_credit_data_updated(
                                                                    ev4.values,
                                                                );
                                                            match event_data {
                                                                Ok(event_data_inner) => {
                                                                    let record = EventRecord {
                                                                        pallet: pallet_name,
                                                                        name: event_name,
                                                                        info: event_data_inner,
                                                                    };
                                                                    res.push(record);
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    res
}

fn decode_credit_data_added(values: Composite) -> Result<EventInfo, Box<dyn std::error::Error>> {
    match values {
        Composite::Unnamed(ev1) => {
            if ev1.len() == 2 {
                let account_id: Result<AccountId32, Box<dyn std::error::Error>> =
                    match ev1[0].clone() {
                        Value::Composite(Composite::Unnamed(ids)) => match ids[0].clone() {
                            Value::Composite(Composite::Unnamed(ids1)) => {
                                crate::common::decode_account_id(ids1)
                            }
                            _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                        },
                        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                    };

                let event_data: Result<CreditData, Box<dyn std::error::Error>> =
                    match ev1[1].clone() {
                        Value::Composite(Composite::Named(ev2)) => {
                            // println!("credit ev2, {:?}, {}", ev2, ev2.len());
                            // TODO: match other fields here
                            let mut credit = 0;
                            for ev2i in &ev2 {
                                // println!("{}, {:?}", ev2i.0, ev2i.1);
                                if ev2i.0 == "credit" {
                                    match ev2i.1.clone() {
                                        Value::Primitive(value::Primitive::U64(inner)) => {
                                            credit = inner;
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Ok(CreditData { credit })
                        }
                        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                    };

                if account_id.is_ok() && event_data.is_ok() {
                    return Ok(EventInfo::CreditDataAdded(
                        account_id.unwrap(),
                        event_data.unwrap(),
                    ));
                }

                Err(Box::new(crate::DecodeAccountIdFailed))
            } else {
                Err(Box::new(crate::DecodeAccountIdFailed))
            }
        }
        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
    }
}

// TODO: fix redudant code
fn decode_credit_data_updated(values: Composite) -> Result<EventInfo, Box<dyn std::error::Error>> {
    match values {
        Composite::Unnamed(ev1) => {
            if ev1.len() == 2 {
                let account_id: Result<AccountId32, Box<dyn std::error::Error>> =
                    match ev1[0].clone() {
                        Value::Composite(Composite::Unnamed(ids)) => match ids[0].clone() {
                            Value::Composite(Composite::Unnamed(ids1)) => {
                                crate::common::decode_account_id(ids1)
                            }
                            _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                        },
                        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                    };

                let event_data: Result<CreditData, Box<dyn std::error::Error>> =
                    match ev1[1].clone() {
                        Value::Composite(Composite::Named(ev2)) => {
                            // println!("credit ev2, {:?}, {}", ev2, ev2.len());
                            // TODO: match other fields here
                            let mut credit = 0;
                            for ev2i in &ev2 {
                                // println!("{}, {:?}", ev2i.0, ev2i.1);
                                if ev2i.0 == "credit" {
                                    match ev2i.1.clone() {
                                        Value::Primitive(value::Primitive::U64(inner)) => {
                                            credit = inner;
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Ok(CreditData { credit })
                        }
                        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
                    };

                if account_id.is_ok() && event_data.is_ok() {
                    return Ok(EventInfo::CreditDataUpdated(
                        account_id.unwrap(),
                        event_data.unwrap(),
                    ));
                }

                Err(Box::new(crate::DecodeAccountIdFailed))
            } else {
                Err(Box::new(crate::DecodeAccountIdFailed))
            }
        }
        _ => Err(Box::new(crate::DecodeAccountIdFailed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::crypto::Ss58Codec;

    #[test]
    fn test_decode_event_credit_data_added() {
        let records = decode_event("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7", "1c0000000000000098e14009000000000200000001000000000000e1f5050000000002000000020000000508d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d8fe676080000000000000000000000000000020000001403a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e01006400000000000000010000000000010e0100000000020000002900000000020000000507d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d8fe6760800000000000000000000000000000200000000001027000000000000000000", crate::deeper_metadata());

        assert_eq!(
            records[0],
            EventRecord {
                pallet: String::from("Credit"),
                name: String::from("CreditDataAdded"),
                info: EventInfo::CreditDataAdded(
                    AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
                        .unwrap(),
                    CreditData { credit: 100 }
                ),
            }
        );

        let event: EventRecord = serde_json::from_str(
            r##"{
            "pallet": "Credit",
            "name": "CreditDataAdded",
            "info": {
                "CreditDataAdded": [
                    "5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ",
                    {
                        "credit": 100
                    }
                ]
            }
        }"##,
        )
        .unwrap();
        assert_eq!(records[0], event);
    }

    #[test]
    fn test_decode_event_credit_data_updated() {
        let records = decode_event("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7", "1c0000000000000098e14009000000000200000001000000000000e1f5050000000002000000020000000508d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d753e45120000000000000000000000000000020000001404a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e01009001000000000000030000000000040e0100000000020000002900000000020000000507d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d753e451200000000000000000000000000000200000000008828ff0900000000000000", crate::deeper_metadata());

        assert_eq!(
            records[0],
            EventRecord {
                pallet: String::from("Credit"),
                name: String::from("CreditDataUpdated"),
                info: EventInfo::CreditDataUpdated(
                    AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
                        .unwrap(),
                    CreditData { credit: 400 }
                ),
            }
        );

        let event: EventRecord = serde_json::from_str(
            r##"{
            "pallet": "Credit",
            "name": "CreditDataUpdated",
            "info": {
                "CreditDataUpdated": [
                    "5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ",
                    {
                        "credit": 400
                    }
                ]
            }
        }"##,
        )
        .unwrap();
        assert_eq!(records[0], event);
    }
}
