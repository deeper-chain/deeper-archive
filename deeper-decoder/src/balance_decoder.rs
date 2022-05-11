use desub_current::value::{Composite, Value};
use sp_core::crypto::AccountId32;
use sp_runtime::MultiAddress;
use std::collections::HashSet;

pub fn get_balance_changed_account_ids(ext: &str) -> HashSet<AccountId32> {
    let mut account_ids: HashSet<AccountId32> = HashSet::new();

    match serde_json::from_str::<Vec<crate::CurrentExtrinsic>>(ext) {
        Ok(extrinsics) => {
            for extrinsic in &extrinsics {
                if extrinsic.current.call_data.pallet_name == "Balances"
                    && ["transfer", "transfer_keep_alive", "transfer_all"]
                        .contains(&extrinsic.current.call_data.ty.name().as_str())
                {
                    if extrinsic.current.call_data.arguments.len() > 1 {
                        match extrinsic.current.call_data.arguments[0].clone() {
                            Value::Composite(Composite::Named(cn)) => {
                                // println!("cn, len: {}, val: {:?}", cn.len(), cn[1]);
                                match cn[1].1.clone() {
                                    Value::Composite(Composite::Unnamed(cn1)) => {
                                        // println!("cn1 {:?}, {}", cn1[0], cn1.len());
                                        match cn1[0].clone() {
                                            Value::Composite(Composite::Unnamed(cn2)) => {
                                                // println!("cn2 {:?}", cn2);
                                                match cn2[0].clone() {
                                                    Value::Composite(Composite::Unnamed(cn3)) => {
                                                        match crate::common::decode_account_id(cn3)
                                                        {
                                                            Ok(account) => {
                                                                account_ids.insert(account);
                                                            }
                                                            _ => {} // ignore error
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                };
                            }
                            _ => {}
                        };
                    }
                }

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
            return account_ids;
        }
        Err(_) => {
            return account_ids;
        }
    }
}

#[cfg(test)]
mod tests {
    use sp_core::crypto::Ss58Codec;

    use super::*;

    #[test]
    fn test_transfer() {
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
                    1649339555002
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
                      "Transfer some liquid free balance to another account.",
                      "",
                      "`transfer` will set the `FreeBalance` of the sender and receiver.",
                      "If the sender's account is below the existential deposit as a result",
                      "of the transfer, the account will be reaped.",
                      "",
                      "The dispatch origin for this call must be `Signed` by the transactor.",
                      "",
                      "# <weight>",
                      "- Dependent on arguments but not critical, given proper implementations for input config",
                      "  types. See related functions below.",
                      "- It contains a limited number of reads and writes internally and no complex",
                      "  computation.",
                      "",
                      "Related functions:",
                      "",
                      "  - `ensure_can_withdraw` is always called internally but has a bounded complexity.",
                      "  - Transferring balances to accounts that did not exist before will cause",
                      "    `T::OnNewAccount::on_new_account` to be called.",
                      "  - Removing enough funds from an account will trigger `T::DustRemoval::on_unbalanced`.",
                      "  - `transfer_keep_alive` works the same way as `transfer`, but has an additional check",
                      "    that the transfer will not kill the origin account.",
                      "---------------------------------",
                      "- Origin account is already in memory, so no DB operations for them.",
                      "# </weight>"
                    ],
                    "name": "transfer",
                    "index": 0,
                    "fields": [
                      {
                        "name": "dest",
                        "type": 155,
                        "typeName": "<T::Lookup as StaticLookup>::Source"
                      },
                      {
                        "name": "value",
                        "type": 53,
                        "typeName": "T::Balance"
                      }
                    ]
                  },
                  "arguments": [
                    {
                      "name": "Id",
                      "values": [
                        [
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
                        ]
                      ]
                    },
                    11110000000000
                  ],
                  "pallet_name": "Balances"
                },
                "signature": {
                  "address": {
                    "Id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                  },
                  "signature": {
                    "Sr25519": "08b5ad39e23c1fb31d0ae0b75e5257a3a0d05fb004a0dd3930ad5ab9a258880f4389f5ea6c15da8ecf35f1d0742568e00938bd47d3efd0c60104972960f03483"
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
                          "name": "Mortal246",
                          "values": [
                            6
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
        let account_ids = get_balance_changed_account_ids(s);
        let dest = AccountId32::from_ss58check("5FshJD1E8MuZw4U2sUWLQHeKuDmkQ85MZacBA36PEJj77xAZ")
            .unwrap();

        assert!(account_ids.contains(&dest));
    }

    #[test]
    fn test_transfer_keep_alive() {
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
                      "type": 126,
                      "typeName": "T::Moment"
                    }
                  ]
                },
                "arguments": [
                  1639722390001
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
                  "docs": [
                    "Same as the [`transfer`] call, but with a check that the transfer will not kill the",
                    "origin account.",
                    "",
                    "99% of the time you want [`transfer`] instead.",
                    "",
                    "[`transfer`]: struct.Pallet.html#method.transfer",
                    "# <weight>",
                    "- Cheaper than transfer because account cannot be killed.",
                    "- Base Weight: 51.4 µs",
                    "- DB Weight: 1 Read and 1 Write to dest (sender is in overlay already)",
                    "#</weight>"
                  ],
                  "name": "transfer_keep_alive",
                  "index": 3,
                  "fields": [
                    {
                      "name": "dest",
                      "type": 131,
                      "typeName": "<T::Lookup as StaticLookup>::Source"
                    },
                    {
                      "name": "value",
                      "type": 66,
                      "typeName": "T::Balance"
                    }
                  ]
                },
                "arguments": [
                  {
                    "name": "Id",
                    "values": [
                      [
                        [
                          56,
                          243,
                          222,
                          30,
                          247,
                          37,
                          146,
                          39,
                          200,
                          98,
                          97,
                          55,
                          28,
                          130,
                          91,
                          139,
                          193,
                          6,
                          244,
                          36,
                          44,
                          242,
                          75,
                          40,
                          210,
                          142,
                          131,
                          30,
                          244,
                          180,
                          142,
                          76
                        ]
                      ]
                    ]
                  },
                  50000000000000000000
                ],
                "pallet_name": "Balances"
              },
              "signature": {
                "address": {
                  "Id": "5CHu6tEdZWEnGHa928e9CfsXnL5otzRg4xGwqCscXDrcH38t"
                },
                "signature": {
                  "Sr25519": "6ea65968e3da6c526934007ca9cddc012ba59f4554a00742ac07e5b95e4828580de223d3ff02efe74f4f5c05753eaa3d25fe0e2567348428a99ecb4bf925898f"
                },
                "extensions": [
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
                        "name": "Mortal134",
                        "values": [
                          0
                        ]
                      }
                    ]
                  ],
                  [
                    "CheckNonce",
                    [
                      8
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
        let account_ids = get_balance_changed_account_ids(s);
        let arr: Vec<AccountId32> = account_ids.iter().cloned().collect();
        println!("{}, {}", arr[0].to_ss58check(), arr[1].to_ss58check());
        let exp = AccountId32::from_ss58check("5DMNywVCTpuVUkoiitr5fYc9QT8ses5Rk4vTyQjfA5Lma5dr")
            .unwrap();

        let alice = AccountId32::from_ss58check("5CHu6tEdZWEnGHa928e9CfsXnL5otzRg4xGwqCscXDrcH38t")
            .unwrap();

        assert!(account_ids.contains(&exp));
        assert!(account_ids.contains(&alice));
        assert_eq!(2, account_ids.len());
    }

    #[test]
    fn test_legacy_extrinsics() {
        let s = r##"[
        {
          "Legacy": {
            "call": {
              "args": [
                {
                  "arg": 1639713065000,
                  "name": "now"
                }
              ],
              "name": "set",
              "module": "Timestamp"
            },
            "signature": null
          }
        },
        {
          "Legacy": {
            "call": {
              "args": [
                {
                  "arg": [
                    [
                      "code",
                      "0x0061736d01000..."
                    ]
                  ],
                  "name": "call"
                },
                {
                  "arg": 1000000,
                  "name": "_weight"
                }
              ],
              "name": "set_code",
              "module": "System"
            },
            "signature": {
              "extra": [
                null,
                null,
                null,
                {
                  "Mortal": [
                    128,
                    119
                  ]
                },
                0,
                null,
                0
              ],
              "address": "5CHu6tEdZWEnGHa928e9CfsXnL5otzRg4xGwqCscXDrcH38t",
              "signature": {
                "name": "Sr25519",
                "value": "0xf2bc2aa7be0920a938aafa71b290e3a74f8d9bc7ab23aa51c5627fd8340a2556e9a027aec3e70773bf57d0585895cdb98764cd826f2e8a4f28c39cfbac50488d"
              }
            }
          }
        }
      ]"##;

        let account_ids = get_balance_changed_account_ids(s);

        // this block is upgrade to spec 7, the set_code extrinsics is legacy, so can't be decoded
        assert!(account_ids.is_empty());
    }
}
