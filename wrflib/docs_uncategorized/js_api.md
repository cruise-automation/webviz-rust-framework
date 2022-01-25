# Wrflib JS API

| API                                         | browser top-level | browser Web Worker | native top-level | native Web Worker |
| ------------------------------------------- | :---------------: | :---------------: | :--------------: | :--------------: |
| wrflib.initialize                           |       ✅          |        —          |       ✅         |       —         |
| wrflib.initializeWorker                     |        —          |        ✅          |       —         |       ❌         |
| wrflib.registerCallJsCallbacks              |       ✅          |        —          |       ✅         |       —         |
| wrflib.unregisterCallJsCallbacks            |       ✅          |        —          |       ✅         |       —         |
| wrflib.callRust                             |       ✅          |        ✅          |       ✅         |       ❌         |
| wrflib.createReadOnlyBuffer                 |       ✅          |        ✅          |       ✅         |       ❌         |
| wrflib.createMutableBuffer                  |       ✅          |        ✅          |       ✅         |       ❌         |
| wrflib.callRustInSameThreadSync             |       —          |        ✅          |       ✅         |       ❌         |
| wrflib.newWorkerPort                     |       ✅          |        ✅          |       ❌         |       ❌         |
| wrflib.serializeWrfArrayForPostMessage      |       ✅          |        ✅          |       ❌         |       ❌         |
| wrflib.deserializeWrfArrayFromPostMessage   |       ✅          |        ✅          |       ❌         |       ❌         |
| wrflib.jsRuntime                            |       ✅          |        ❌          |       ✅         |       ❌         |

```
✅ = implemented
❌ = TODO
—  = not applicable
```
