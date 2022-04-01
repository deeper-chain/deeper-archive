use desub_current::decoder::Extrinsic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentExtrinsic<'a> {
    #[serde(borrow)]
    #[serde(rename = "Current")]
    pub current: Extrinsic<'a>,
}
fn main() {
    let data = r##"[
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
                1648640270002
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
                  "Same as the [`transfer`] call, but with a check that the transfer will not kill the",
                  "origin account.",
                  "",
                  "99% of the time you want [`transfer`] instead.",
                  "",
                  "[`transfer`]: struct.Pallet.html#method.transfer"
                ],
                "name": "transfer_keep_alive",
                "index": 3,
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
                100000000000000000000
              ],
              "pallet_name": "Balances"
            },
            "signature": {
              "address": {
                "Id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
              },
              "signature": {
                "Sr25519": "c2f5db2900a47b8bf82416c39dce8a20a0a75eb95c5c02ca7e436885ca696721c9b0a7e56598abaf77351fe23ab39e74741d08bffcc05f3fcf3f32e7d778af84"
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

    let p: Vec<CurrentExtrinsic> = serde_json::from_str(data).unwrap();

    println!("Please call {:?}", p[2].current.call_data.arguments[1]);
}
