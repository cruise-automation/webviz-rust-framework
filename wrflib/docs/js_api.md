# Wrf JS API

| API                                         | browser top-level | browser WebWorker | native top-level | native WebWorker |
| ------------------------------------------- | :---------------: | :---------------: | :--------------: | :--------------: |
| wrf/self.initialize                         |       ✅          |        —          |       ✅         |       —         |
| wrf/self.initWrfUserWorkerRuntime           |       —          |        ✅          |       —         |       ❌         |
| wrf/self.wrfInitialized                     |       ❌          |        ✅          |       ❌         |       ❌         |
| wrf/self.registerCallJsCallbacks            |       ✅          |        —          |       ✅         |       —         |
| wrf/self.unregisterCallJsCallbacks          |       ✅          |        —          |       ✅         |       —         |
| wrf/self.callRust                           |       ✅          |        ✅          |       ✅         |       ❌         |
| wrf/self.createBuffer                       |       ✅          |        ✅          |       ✅         |       ❌         |
| wrf/self.createReadOnlyBuffer               |       ✅          |        ✅          |       ✅         |       ❌         |
| wrf/self.callRustInSameThreadSync           |       —          |        ✅          |       ✅         |       ❌         |
| wrf/self.wrfNewWorkerPort                   |       ✅          |        ✅          |       ❌         |       ❌         |
| wrf/self.serializeWrfArrayForPostMessage    |       ✅          |        ✅          |       ❌         |       ❌         |
| wrf/self.deserializeWrfArrayFromPostMessage |       ✅          |        ✅          |       ❌         |       ❌         |
| wrf/self.isWrfBuffer                        |       ❌          |        ✅          |       ❌         |       ❌         |
| wrf/self.jsRuntime                          |       ✅          |        ❌          |       ✅         |       ❌         |

```
✅ = implemented
❌ = TODO
—  = not applicable
```
