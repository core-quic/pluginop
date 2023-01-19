window.BENCHMARK_DATA = {
  "lastUpdate": 1674144510141,
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
      }
    ]
  }
}