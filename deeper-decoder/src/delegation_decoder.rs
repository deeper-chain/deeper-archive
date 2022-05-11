use desub_current::value::{Composite, Value};
use desub_current::Metadata;
use sp_core::crypto::AccountId32;
use sp_runtime::MultiAddress;
use std::collections::HashSet;

pub fn get_delegation_changed_account_ids(ext: &str) -> HashSet<AccountId32> {
    let mut account_ids: HashSet<AccountId32> = HashSet::new();

    match serde_json::from_str::<Vec<crate::CurrentExtrinsic>>(ext) {
        Ok(extrinsics) => {
            for extrinsic in &extrinsics {
                if extrinsic.current.call_data.pallet_name == "Staking"
                    && ["delegate", "undelegate"]
                        .contains(&extrinsic.current.call_data.ty.name().as_str())
                {
                    match extrinsic.current.signature.clone() {
                        Some(signature_val) => match signature_val.address {
                            MultiAddress::Id(account_id) => {
                                account_ids.insert(account_id);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            account_ids
        }
        Err(_) => account_ids,
    }
}

pub fn get_validators(storage_key: &str, storage_val: &str, meta: &Metadata) -> Vec<AccountId32> {
    let val = crate::common::decode_storage(storage_key, storage_val, meta);
    let mut res = vec![];
    match val {
        Value::Composite(Composite::Named(cn)) => {
            for cn1 in &cn {
                if cn1.0 == "delegated_validators" {
                    match cn1.1.clone() {
                        Value::Composite(Composite::Unnamed(cn2)) => match cn2[0].clone() {
                            Value::Composite(Composite::Unnamed(cn3)) => {
                                for cn3i in cn3 {
                                    match cn3i.clone() {
                                        Value::Composite(Composite::Unnamed(cn4)) => {
                                            let addrs_result =
                                                crate::common::decode_account_id(cn4.clone());
                                            match addrs_result {
                                                Ok(account_id) => res.push(account_id),
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {
                            return res;
                        }
                    }
                }
            }
            res
        }
        _ => res,
    }
}

#[cfg(test)]
mod tests {
    use sp_core::crypto::Ss58Codec;

    use crate::common::deeper_metadata;

    use super::*;

    #[test]
    fn test_get_delegator() {
        let res = get_validators("5f3e4907f716ac89b6347d15ececedcae1c5df6d2773f08c7b6b1b6d0139c22a3594ef778a4003043f6d977057644d65a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e", "a88b59afe73f0e769e4f9d85cd40fd13f0874446f22d2ab6780f9cb89059307e04be5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f010100000001", &deeper_metadata());
        let alice_stash =
            AccountId32::from_ss58check("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")
                .unwrap();
        assert_eq!(res, vec![alice_stash]);
    }

    #[test]
    fn test_delegate() {
        let s = r##"[
            {
              "Current": {
                "call_data": {
                  "ty": {
                    "docs": [
                      "Set the current time.",
                      "",
                      "This call should be invoked exactly once per block. It will panic at the finalization",
                      "phase, if this call hasn't been invoked by that time.",
                      "",
                      "The timestamp should be greater than the previous one by the amount specified by",
                      "`MinimumPeriod`.",
                      "",
                      "The dispatch origin for this call must be `Inherent`.",
                      "",
                      "# <weight>",
                      "- `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)",
                      "- 1 storage read and 1 storage mutation (codec `O(1)`). (because of `DidUpdate::take` in",
                      "  `on_finalize`)",
                      "- 1 event handler `on_timestamp_set`. Must be `O(1)`.",
                      "# </weight>"
                    ],
                    "name": "set",
                    "index": 0,
                    "fields": [
                      {
                        "name": "now",
                        "type": 152,
                        "typeName": "T::Moment"
                      }
                    ]
                  },
                  "arguments": [
                    1649861160001
                  ],
                  "pallet_name": "Timestamp"
                },
                "signature": null
              }
            },
            {
              "Current": {
                "call_data": {
                  "ty": {
                    "name": "note_min_gas_price_target",
                    "index": 0,
                    "fields": [
                      {
                        "name": "target",
                        "type": 106,
                        "typeName": "U256"
                      }
                    ]
                  },
                  "arguments": [
                    [
                      [
                        1,
                        0,
                        0,
                        0
                      ]
                    ]
                  ],
                  "pallet_name": "DynamicFee"
                },
                "signature": null
              }
            },
            {
              "Current": {
                "call_data": {
                  "ty": {
                    "docs": [
                      "delegate credit to a set of validators"
                    ],
                    "name": "delegate",
                    "index": 23,
                    "fields": [
                      {
                        "name": "validators",
                        "type": 38,
                        "typeName": "Vec<T::AccountId>"
                      }
                    ]
                  },
                  "arguments": [
                    [
                      [
                        [
                          190,
                          93,
                          219,
                          21,
                          121,
                          183,
                          46,
                          132,
                          82,
                          79,
                          194,
                          158,
                          120,
                          96,
                          158,
                          60,
                          175,
                          66,
                          232,
                          90,
                          161,
                          24,
                          235,
                          254,
                          11,
                          10,
                          212,
                          4,
                          181,
                          189,
                          210,
                          95
                        ]
                      ]
                    ]
                  ],
                  "pallet_name": "Staking"
                },
                "signature": {
                  "address": {
                    "Id": "5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ"
                  },
                  "signature": {
                    "Sr25519": "989bc770140c994dc5b9d1e63928b250936a8d6c374dc0e65666271e8e0df12a6a128c6872224a39a25d2f1c0777202cabfec15339de22160d3b786405a20f80"
                  },
                  "extensions": [
                    [
                      "CheckNonZeroSender",
                      []
                    ],
                    [
                      "CheckSpecVersion",
                      []
                    ],
                    [
                      "CheckTxVersion",
                      []
                    ],
                    [
                      "CheckGenesis",
                      []
                    ],
                    [
                      "CheckMortality",
                      [
                        {
                          "name": "Mortal86",
                          "values": [
                            7
                          ]
                        }
                      ]
                    ],
                    [
                      "CheckNonce",
                      [
                        2
                      ]
                    ],
                    [
                      "CheckWeight",
                      []
                    ],
                    [
                      "ChargeTransactionPayment",
                      [
                        0
                      ]
                    ]
                  ]
                }
              }
            }
          ]"##;
        let account_ids = get_delegation_changed_account_ids(s);
        let dest = AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
            .unwrap();

        assert!(account_ids.contains(&dest));
    }
}
