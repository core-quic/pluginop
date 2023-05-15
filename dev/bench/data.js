window.BENCHMARK_DATA = {
  "lastUpdate": 1684159591963,
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
          "id": "e45b74cc51725fa758d7c1404acde1d98bac04d4",
          "message": "first working version for the simplest plugin of core-quic",
          "timestamp": "2023-03-14T18:29:05+01:00",
          "tree_id": "6ca17fa349746661dd56f4386b30a47086187ad6",
          "url": "https://github.com/qdeconinck/pluginop/commit/e45b74cc51725fa758d7c1404acde1d98bac04d4"
        },
        "date": 1678815345396,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 140,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 5995033,
            "range": "± 550361",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4369,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4840,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5956,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4065,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11900884,
            "range": "± 266609",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5764107,
            "range": "± 122491",
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
          "id": "ad8d4e2b035423b997fbc645600dc882bdbaa31a",
          "message": "Merge pull request #20 from qdeconinck/bytes-api\n\nA first working version of the Bytes API",
          "timestamp": "2023-03-22T12:16:50+01:00",
          "tree_id": "059f175c4d2d4abe1e30c1fc369d8ea851a31e95",
          "url": "https://github.com/qdeconinck/pluginop/commit/ad8d4e2b035423b997fbc645600dc882bdbaa31a"
        },
        "date": 1679484197639,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 149,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 6328310,
            "range": "± 171686",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4288,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4765,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5874,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4391,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11411924,
            "range": "± 516969",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5708782,
            "range": "± 251173",
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
          "id": "320a7427b632d7283163b3c2822457763450bb87",
          "message": "Merge pull request #21 from qdeconinck/super-frame\n\nhuge rewriting + refactoring that works with core-quiche",
          "timestamp": "2023-03-29T11:34:52+02:00",
          "tree_id": "7062171ff36b2834a2d92f732e999a8bf4dd304c",
          "url": "https://github.com/qdeconinck/pluginop/commit/320a7427b632d7283163b3c2822457763450bb87"
        },
        "date": 1680082900231,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 140,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 8908797,
            "range": "± 1658687",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4235,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4638,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6015,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4411,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11725558,
            "range": "± 433537",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5772391,
            "range": "± 249056",
            "unit": "ns/iter"
          },
          {
            "name": "max-data",
            "value": 262,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame",
            "value": 11453,
            "range": "± 551",
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
          "id": "7014a4022dbc9c4319331dc31ce9b62c9f704446",
          "message": "Merge pull request #22 from qdeconinck/store-per-plugin\n\nStore is per-plugin",
          "timestamp": "2023-03-30T12:28:09+02:00",
          "tree_id": "7ef33431c090eb9a23d48e081bfec02141e4cc9c",
          "url": "https://github.com/qdeconinck/pluginop/commit/7014a4022dbc9c4319331dc31ce9b62c9f704446"
        },
        "date": 1680172496114,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 153,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 6583545,
            "range": "± 1646725",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4355,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4973,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6024,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4471,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11265392,
            "range": "± 511563",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5661990,
            "range": "± 241538",
            "unit": "ns/iter"
          },
          {
            "name": "max-data",
            "value": 279,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame",
            "value": 11832,
            "range": "± 549",
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
          "id": "42dc2a1c2732a1788bdb19d2b1a25fd2fe7ec00d",
          "message": "add new test doing the same as native mock\n\n+ support for flamegraph benchmarking.",
          "timestamp": "2023-03-30T15:55:49+02:00",
          "tree_id": "69578b4f768a0272941f27a5654b86fb9d6d5548",
          "url": "https://github.com/qdeconinck/pluginop/commit/42dc2a1c2732a1788bdb19d2b1a25fd2fe7ec00d"
        },
        "date": 1680185050932,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 191,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7452510,
            "range": "± 394735",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4921,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5611,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6972,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 5051,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 13515821,
            "range": "± 693261",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 6628502,
            "range": "± 590096",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 341,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 9428,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 15402,
            "range": "± 817",
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
          "id": "76090615798fea262a1d4dacf658e4e0d2341897",
          "message": "initial version of README",
          "timestamp": "2023-03-31T17:12:18+02:00",
          "tree_id": "799b5e3f649ba1f9cb0576546327658631445a3e",
          "url": "https://github.com/qdeconinck/pluginop/commit/76090615798fea262a1d4dacf658e4e0d2341897"
        },
        "date": 1680276024692,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 161,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 9821246,
            "range": "± 1799427",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4860,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5133,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6807,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 5192,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 13873683,
            "range": "± 697981",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 6323248,
            "range": "± 595196",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 273,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 8929,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 14098,
            "range": "± 922",
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
          "id": "08e6b064fd43f076e73c0cf837ed45db30f15055",
          "message": "fix codecov badge",
          "timestamp": "2023-03-31T17:14:19+02:00",
          "tree_id": "32afa27c4a2cfbe521b3e41b3b6f2ae69afbf982",
          "url": "https://github.com/qdeconinck/pluginop/commit/08e6b064fd43f076e73c0cf837ed45db30f15055"
        },
        "date": 1680276075885,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 141,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 7998598,
            "range": "± 448576",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4546,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5233,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 6779,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4965,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11994377,
            "range": "± 168984",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5885473,
            "range": "± 177034",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 227,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 9131,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 12596,
            "range": "± 516",
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
          "id": "dcba5d96d0f6b2227e01248cc4702214abd16587",
          "message": "Update README.md",
          "timestamp": "2023-04-04T14:20:38+02:00",
          "tree_id": "68321591d880a849c80aace87d69d5e3fcc58813",
          "url": "https://github.com/qdeconinck/pluginop/commit/dcba5d96d0f6b2227e01248cc4702214abd16587"
        },
        "date": 1680611332727,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 176,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 12072176,
            "range": "± 2005541",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 5079,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5794,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 7982,
            "range": "± 582",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 5566,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 15687634,
            "range": "± 651110",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 7365591,
            "range": "± 325873",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 300,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 10185,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 15108,
            "range": "± 1108",
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
          "id": "61b2238056928e98e8ca3103b29437c52e66b678",
          "message": "Merge pull request #23 from qdeconinck/timer-support\n\ntimer support",
          "timestamp": "2023-05-12T12:35:44+02:00",
          "tree_id": "11125890455d4328c7adfc34412869e27186be5e",
          "url": "https://github.com/qdeconinck/pluginop/commit/61b2238056928e98e8ca3103b29437c52e66b678"
        },
        "date": 1683888181657,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 155,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 5466796,
            "range": "± 1418908",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 3974,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 4381,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 5405,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 4137,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 9747262,
            "range": "± 795447",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 4716186,
            "range": "± 313482",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 240,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 7082,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 10013,
            "range": "± 803",
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
          "id": "260474abe741137d05400605dc96a078f96536dd",
          "message": "Merge pull request #24 from qdeconinck/hide-unix-time\n\nhide the UNIX-based `Instant` from the host",
          "timestamp": "2023-05-15T15:58:14+02:00",
          "tree_id": "0181713cd239b15f4fd4c8bc2908cdd7754b9bc2",
          "url": "https://github.com/qdeconinck/pluginop/commit/260474abe741137d05400605dc96a078f96536dd"
        },
        "date": 1684159590755,
        "tool": "cargo",
        "benches": [
          {
            "name": "run and return",
            "value": 198,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "memory allocation",
            "value": 4861526,
            "range": "± 167474",
            "unit": "ns/iter"
          },
          {
            "name": "static memory",
            "value": 4894,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "inputs support",
            "value": 5467,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "input outputs",
            "value": 7183,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "increase-max-data",
            "value": 5292,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "first pluginop",
            "value": 11896551,
            "range": "± 336539",
            "unit": "ns/iter"
          },
          {
            "name": "macro simple",
            "value": 5657325,
            "range": "± 290617",
            "unit": "ns/iter"
          },
          {
            "name": "max-data send and receive",
            "value": 289,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "max-data wasm send and receive",
            "value": 8692,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "super-frame send and receive",
            "value": 12336,
            "range": "± 642",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}