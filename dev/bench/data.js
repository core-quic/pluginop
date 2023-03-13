window.BENCHMARK_DATA = {
  "lastUpdate": 1678709361705,
  "repoUrl": "https://github.com/qdeconinck/pluginop",
  "entries": {
    "Pluginop benchmarks": [
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a859316cd1e98467afc69b487cebd0e03a4bc4a8",
          "message": "Merge pull request #5 from qdeconinck/better-input-perfs\n\nserialize inputs directly in WASM memory",
          "timestamp": "2023-01-19T15:26:40+01:00",
          "tree_id": "436cd60d10a9592111b8db63b1f74c679936a41c",
          "url": "https://github.com/qdeconinck/pluginop/commit/a859316cd1e98467afc69b487cebd0e03a4bc4a8"
        },
        "date": 1674138861272,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 570,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 3687859,
            "range": "± 288933",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 7368,
            "range": "± 596",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5dde82a9904b453a32b6ef9c7bae43c14e159d8f",
          "message": "Merge pull request #6 from qdeconinck/inputs-support\n\nadd support to get all inputs in one call + support for print",
          "timestamp": "2023-01-19T17:02:32+01:00",
          "tree_id": "71a34127a271cbcd118b3f26269fd1003be8ec20",
          "url": "https://github.com/qdeconinck/pluginop/commit/5dde82a9904b453a32b6ef9c7bae43c14e159d8f"
        },
        "date": 1674144508637,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 445,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 4075846,
            "range": "± 754959",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 3796,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4127,
            "range": "± 18",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "adfcd9dc0cc74e31b160218edc0ce56dc394d9ca",
          "message": "Merge pull request #7 from qdeconinck/outputs-support\n\nimplemented support for outputs",
          "timestamp": "2023-01-20T11:31:31+01:00",
          "tree_id": "df50de6c23dfe57456892c667fc55883a438a268",
          "url": "https://github.com/qdeconinck/pluginop/commit/adfcd9dc0cc74e31b160218edc0ce56dc394d9ca"
        },
        "date": 1674211049937,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 452,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 5647320,
            "range": "± 1365449",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4888,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5346,
            "range": "± 10",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7bfa7fd43a88fe35077a64623cb40aa5137c27c1",
          "message": "Merge pull request #8 from qdeconinck/typed-functions\n\nuse TypedFunctions to benefit from the ABI",
          "timestamp": "2023-01-20T16:18:28+01:00",
          "tree_id": "bfebce80580495d67a8d650be7d7da187dc0b7f3",
          "url": "https://github.com/qdeconinck/pluginop/commit/7bfa7fd43a88fe35077a64623cb40aa5137c27c1"
        },
        "date": 1674228336212,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 342,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7607627,
            "range": "± 1501618",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4307,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4699,
            "range": "± 203",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "004ababef66ac4690c251e8880021da18915d7ca",
          "message": "Merge pull request #9 from qdeconinck/week-refactoring\n\nrefactor this week's code",
          "timestamp": "2023-01-20T17:46:52+01:00",
          "tree_id": "a3c08f1ad2eb0e7f688a3ee30356e33e9f5fb49d",
          "url": "https://github.com/qdeconinck/pluginop/commit/004ababef66ac4690c251e8880021da18915d7ca"
        },
        "date": 1674233639622,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 339,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 6655328,
            "range": "± 496508",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4735,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5149,
            "range": "± 32",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9c7c2084fcba71810144ac7b2bbd463233ade4dc",
          "message": "Merge pull request #12 from qdeconinck/initial-connection-api\n\ninitial support for the Connection API",
          "timestamp": "2023-01-24T17:01:27+01:00",
          "tree_id": "7da6527e894e929cd9dd3346e8b6612213f081dc",
          "url": "https://github.com/qdeconinck/pluginop/commit/9c7c2084fcba71810144ac7b2bbd463233ade4dc"
        },
        "date": 1674576483184,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 265,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7351683,
            "range": "± 1342111",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 3874,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4774,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 4868,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 3940,
            "range": "± 9",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b61d50a1d82a8439c7e13cf2003a4a5f4fa87071",
          "message": "Merge pull request #13 from qdeconinck/type-alias\n\nAdd type alias for WASM types",
          "timestamp": "2023-01-24T18:07:31+01:00",
          "tree_id": "4bbec31385d23efdaef5ae881d087a318659d889",
          "url": "https://github.com/qdeconinck/pluginop/commit/b61d50a1d82a8439c7e13cf2003a4a5f4fa87071"
        },
        "date": 1674580628500,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 392,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 10169335,
            "range": "± 2125610",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 5948,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 6386,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 8025,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4557,
            "range": "± 208",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0bf9af0b0a371faa22f3c3e5788bb9b352eae731",
          "message": "Merge pull request #14 from qdeconinck/first-pluginop\n\nfirst version working with a hardcoded plugin operation",
          "timestamp": "2023-02-10T11:10:38+01:00",
          "tree_id": "b5c2fd59c9a352df5d10f1bc4174ac5a3da6dd4a",
          "url": "https://github.com/qdeconinck/pluginop/commit/0bf9af0b0a371faa22f3c3e5788bb9b352eae731"
        },
        "date": 1676024245583,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 267,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 9321979,
            "range": "± 1678325",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4842,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5247,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6533,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4311,
            "range": "± 33",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "40d515bfa097294de324ead3d89eb8386f23fe45",
          "message": "Merge pull request #16 from qdeconinck/pluginop-macro\n\nadd pluginop and pluginop_param macros to simplify pluginization",
          "timestamp": "2023-02-10T16:48:14+01:00",
          "tree_id": "f02397958ad4fa50acf74f0eb96cf42ff3af342d",
          "url": "https://github.com/qdeconinck/pluginop/commit/40d515bfa097294de324ead3d89eb8386f23fe45"
        },
        "date": 1676044608738,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 345,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7714774,
            "range": "± 378728",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 5637,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 6175,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 7336,
            "range": "± 574",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4489,
            "range": "± 192",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "24367f9941f15d3915579a5706d2a1d8d36b96f5",
          "message": "Merge pull request #17 from qdeconinck/refactor-test-mock\n\nrefactoring: move mocking code in its own sub-crate",
          "timestamp": "2023-02-13T10:41:29+01:00",
          "tree_id": "769ea030a16091f92b707b8e396741f128d9c4b7",
          "url": "https://github.com/qdeconinck/pluginop/commit/24367f9941f15d3915579a5706d2a1d8d36b96f5"
        },
        "date": 1676281771862,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 323,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7186488,
            "range": "± 645872",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 5153,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5762,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6979,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4359,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 12428403,
            "range": "± 443506",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 6169844,
            "range": "± 255286",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "48f8667b7820e3eb6267de48e96d9bffa1712748",
          "message": "Merge pull request #18 from qdeconinck/pocodes-collection\n\nperf: by default, use a Vec to store plugin functions",
          "timestamp": "2023-02-13T11:31:42+01:00",
          "tree_id": "678402f40b0b3df9002a17753d77261be3542cf6",
          "url": "https://github.com/qdeconinck/pluginop/commit/48f8667b7820e3eb6267de48e96d9bffa1712748"
        },
        "date": 1676284738520,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 142,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 6043602,
            "range": "± 211601",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 3907,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4271,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5365,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 3730,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11143429,
            "range": "± 458286",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5652115,
            "range": "± 161190",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a1025a25183f94c07c469f9a1d6d126275b7e511",
          "message": "Merge pull request #19 from qdeconinck/generic\n\nmake PluginizableConnection a struct",
          "timestamp": "2023-03-13T12:15:34+01:00",
          "tree_id": "9584577f162543aeb0f54ca071c45823e77b1eb4",
          "url": "https://github.com/qdeconinck/pluginop/commit/a1025a25183f94c07c469f9a1d6d126275b7e511"
        },
        "date": 1678706510616,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 142,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 6836835,
            "range": "± 1712718",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4361,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4694,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5930,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4318,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11967430,
            "range": "± 221928",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5758764,
            "range": "± 168552",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "committer": {
            "email": "quentin.deconinck@uclouvain.be",
            "name": "Quentin De Coninck",
            "username": "qdeconinck"
          },
          "distinct": true,
          "id": "b223d37c9027155896e8cde1c28a970fa2931e55",
          "message": "make sub-connection structures reference the PluginizableConnection\n\nBy introducing the `ToPluginizableConnection` trait.",
          "timestamp": "2023-03-13T13:02:14+01:00",
          "tree_id": "2030b1de226000ea85faf68ecec5be2364279eb6",
          "url": "https://github.com/qdeconinck/pluginop/commit/b223d37c9027155896e8cde1c28a970fa2931e55"
        },
        "date": 1678709360855,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 154,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 5882584,
            "range": "± 265945",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 3863,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4302,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5490,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 3847,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11342272,
            "range": "± 951299",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5650461,
            "range": "± 195067",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}