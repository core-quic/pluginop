window.BENCHMARK_DATA = {
  "lastUpdate": 1674233641376,
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
      }
    ]
  }
}