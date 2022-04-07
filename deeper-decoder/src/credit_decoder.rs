use desub_current::value::{self, Composite, Value};
use sp_core::crypto::AccountId32;
use std::collections::HashSet;

pub fn get_credit_changed_account_ids(ext: &str) -> HashSet<AccountId32> {
    let extrinsics: Vec<crate::CurrentExtrinsic> = serde_json::from_str(ext).unwrap();

    let mut account_ids: HashSet<AccountId32> = HashSet::new();
    for extrinsic in &extrinsics {
        // TODO: check sudo
        for argument in extrinsic.current.call_data.arguments.clone() {
            match argument {
                Value::Composite(Composite::Named(cn)) => {
                    match cn[1].1.clone() {
                        Value::Composite(Composite::Unnamed(cn1)) => {
                            match cn1[0].clone() {
                                Value::Composite(Composite::Named(cn2)) => {
                                    if cn2[0].0 == "name"
                                        && cn2[0].1
                                            == Value::Primitive(value::Primitive::Str(
                                                String::from("add_or_update_credit_data"),
                                            ))
                                    {
                                        match cn2[1].1.clone() {
                                            Value::Composite(Composite::Named(cn3)) => {
                                                if cn3[0].0 == "account_id" {
                                                    match cn3[0].1.clone() {
                                                        Value::Composite(Composite::Unnamed(
                                                            cn4,
                                                        )) => match cn4[0].clone() {
                                                            Value::Composite(
                                                                Composite::Unnamed(cn5),
                                                            ) => {
                                                                match crate::common::decode_account_id(cn5) {
                                                                  Ok(account) => {
                                                                    account_ids.insert(account);
                                                                  },
                                                                  _ => {}, // ignore error
                                                                }
                                                            }
                                                            _ => {}
                                                        },
                                                        _ => {}
                                                    };
                                                }
                                            }
                                            _ => {}
                                        };
                                    }
                                }
                                _ => {}
                            };
                        }
                        _ => {}
                    };
                }
                _ => {}
            };
        }
    }
    account_ids
}



#[cfg(test)]
mod tests {
    use sp_core::crypto::Ss58Codec;

    use super::*;

    #[test]
    fn test_sudo_add_credit() {
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
          1649323090002
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
            "Authenticates the sudo key and dispatches a function call with `Root` origin.",
            "This function does not check the weight of the call, and instead allows the",
            "Sudo user to specify the weight of the call.",
            "",
            "The dispatch origin for this call must be _Signed_.",
            "",
            "# <weight>",
            "- O(1).",
            "- The weight of this call is defined by the caller.",
            "# </weight>"
          ],
          "name": "sudo_unchecked_weight",
          "index": 1,
          "fields": [
            {
              "name": "call",
              "type": 137,
              "typeName": "Box<<T as Config>::Call>"
            },
            {
              "name": "weight",
              "type": 8,
              "typeName": "Weight"
            }
          ]
        },
        "arguments": [
          {
            "name": "Credit",
            "values": [
              {
                "name": "add_or_update_credit_data",
                "values": {
                  "account_id": [
                    [
                      168,
                      139,
                      89,
                      175,
                      231,
                      63,
                      14,
                      118,
                      158,
                      79,
                      157,
                      133,
                      205,
                      64,
                      253,
                      19,
                      240,
                      135,
                      68,
                      70,
                      242,
                      45,
                      42,
                      182,
                      120,
                      15,
                      156,
                      184,
                      144,
                      89,
                      48,
                      126
                    ]
                  ],
                  "credit_data": {
                    "credit": 100,
                    "campaign_id": 1,
                    "reward_eras": 270,
                    "number_of_referees": 0,
                    "current_credit_level": {
                      "name": "One",
                      "values": []
                    },
                    "initial_credit_level": {
                      "name": "One",
                      "values": []
                    },
                    "rank_in_initial_credit_level": 0
                  }
                }
              }
            ]
          },
          10000
        ],
        "pallet_name": "Sudo"
      },
      "signature": {
        "address": {
          "Id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        },
        "signature": {
          "Sr25519": "50e13f3a4e56b6f9b4bb45a07540219ca739850ac20bd6c1310bc71fb5d35d6c60b8ff77d55b9352124562669aa3d8b3f7cffd309f6709a1aab24f8fa3e9bb80"
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
                "name": "Mortal70",
                "values": [
                  1
                ]
              }
            ]
          ],
          [
            "CheckNonce",
            [
              0
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
        let account_ids = get_credit_changed_account_ids(s);
        let exp = sp_core::crypto::AccountId32::from_ss58check(
            "5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ",
        )
        .unwrap();

        assert!(account_ids.contains(&exp));
    }
}
