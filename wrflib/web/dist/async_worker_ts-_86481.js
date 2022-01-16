/*
 * ATTENTION: An "eval-source-map" devtool has been used.
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file with attached SourceMaps in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
(function webpackUniversalModuleDefinition(root, factory) {
	if(typeof exports === 'object' && typeof module === 'object')
		module.exports = factory();
	else if(typeof define === 'function' && define.amd)
		define([], factory);
	else if(typeof exports === 'object')
		exports["wrf"] = factory();
	else
		root["wrf"] = factory();
})(self, function() {
return /******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./async_worker.ts":
/*!*************************!*\
  !*** ./async_worker.ts ***!
  \*************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _common__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./common */ \"./common.ts\");\n/* harmony import */ var _types__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./types */ \"./types.ts\");\n// Copyright (c) 2021-present, Cruise LLC\n//\n// This source code is licensed under the Apache License, Version 2.0,\n// found in the LICENSE-APACHE file in the root directory of this source tree.\n// You may not use this file except in compliance with the License.\n\n\nconst rpc = new _common__WEBPACK_IMPORTED_MODULE_0__.Rpc(self);\nrpc.receive(_types__WEBPACK_IMPORTED_MODULE_1__.AsyncWorkerEvent.Run, ({ wasmModule, memory, taskWorkerSab, ctxPtr, fileHandles, baseUri, tlsAndStackData, }) => {\n    const sendEventFromAnyThread = (eventPtr) => {\n        rpc.send(_types__WEBPACK_IMPORTED_MODULE_1__.AsyncWorkerEvent.SendEventFromAnyThread, { eventPtr });\n    };\n    const threadSpawn = (ctxPtr) => {\n        rpc.send(_types__WEBPACK_IMPORTED_MODULE_1__.AsyncWorkerEvent.ThreadSpawn, { ctxPtr });\n    };\n    let exports;\n    const getExports = () => {\n        return exports;\n    };\n    const env = (0,_common__WEBPACK_IMPORTED_MODULE_0__.getWasmEnv)({\n        getExports,\n        memory,\n        taskWorkerSab,\n        fileHandles,\n        sendEventFromAnyThread,\n        threadSpawn,\n        baseUri,\n    });\n    return new Promise((resolve, reject) => {\n        WebAssembly.instantiate(wasmModule, { env }).then((instance) => {\n            exports = instance.exports;\n            (0,_common__WEBPACK_IMPORTED_MODULE_0__.initThreadLocalStorageAndStackOtherWorkers)(exports, tlsAndStackData);\n            // TODO(Paras): Eventually call `processWasmEvents` instead of a custom exported function.\n            exports.runFunctionPointer(ctxPtr);\n            resolve();\n        }, reject);\n    });\n});\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9hc3luY193b3JrZXIudHMuanMiLCJtYXBwaW5ncyI6Ijs7O0FBQUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSIsInNvdXJjZXMiOlsid2VicGFjazovL3dyZi8uL2FzeW5jX3dvcmtlci50cz84NjQ4Il0sInNvdXJjZXNDb250ZW50IjpbIi8vIENvcHlyaWdodCAoYykgMjAyMS1wcmVzZW50LCBDcnVpc2UgTExDXG4vL1xuLy8gVGhpcyBzb3VyY2UgY29kZSBpcyBsaWNlbnNlZCB1bmRlciB0aGUgQXBhY2hlIExpY2Vuc2UsIFZlcnNpb24gMi4wLFxuLy8gZm91bmQgaW4gdGhlIExJQ0VOU0UtQVBBQ0hFIGZpbGUgaW4gdGhlIHJvb3QgZGlyZWN0b3J5IG9mIHRoaXMgc291cmNlIHRyZWUuXG4vLyBZb3UgbWF5IG5vdCB1c2UgdGhpcyBmaWxlIGV4Y2VwdCBpbiBjb21wbGlhbmNlIHdpdGggdGhlIExpY2Vuc2UuXG5pbXBvcnQgeyBScGMsIGdldFdhc21FbnYsIGluaXRUaHJlYWRMb2NhbFN0b3JhZ2VBbmRTdGFja090aGVyV29ya2VycywgfSBmcm9tIFwiLi9jb21tb25cIjtcbmltcG9ydCB7IEFzeW5jV29ya2VyRXZlbnQgfSBmcm9tIFwiLi90eXBlc1wiO1xuY29uc3QgcnBjID0gbmV3IFJwYyhzZWxmKTtcbnJwYy5yZWNlaXZlKEFzeW5jV29ya2VyRXZlbnQuUnVuLCAoeyB3YXNtTW9kdWxlLCBtZW1vcnksIHRhc2tXb3JrZXJTYWIsIGN0eFB0ciwgZmlsZUhhbmRsZXMsIGJhc2VVcmksIHRsc0FuZFN0YWNrRGF0YSwgfSkgPT4ge1xuICAgIGNvbnN0IHNlbmRFdmVudEZyb21BbnlUaHJlYWQgPSAoZXZlbnRQdHIpID0+IHtcbiAgICAgICAgcnBjLnNlbmQoQXN5bmNXb3JrZXJFdmVudC5TZW5kRXZlbnRGcm9tQW55VGhyZWFkLCB7IGV2ZW50UHRyIH0pO1xuICAgIH07XG4gICAgY29uc3QgdGhyZWFkU3Bhd24gPSAoY3R4UHRyKSA9PiB7XG4gICAgICAgIHJwYy5zZW5kKEFzeW5jV29ya2VyRXZlbnQuVGhyZWFkU3Bhd24sIHsgY3R4UHRyIH0pO1xuICAgIH07XG4gICAgbGV0IGV4cG9ydHM7XG4gICAgY29uc3QgZ2V0RXhwb3J0cyA9ICgpID0+IHtcbiAgICAgICAgcmV0dXJuIGV4cG9ydHM7XG4gICAgfTtcbiAgICBjb25zdCBlbnYgPSBnZXRXYXNtRW52KHtcbiAgICAgICAgZ2V0RXhwb3J0cyxcbiAgICAgICAgbWVtb3J5LFxuICAgICAgICB0YXNrV29ya2VyU2FiLFxuICAgICAgICBmaWxlSGFuZGxlcyxcbiAgICAgICAgc2VuZEV2ZW50RnJvbUFueVRocmVhZCxcbiAgICAgICAgdGhyZWFkU3Bhd24sXG4gICAgICAgIGJhc2VVcmksXG4gICAgfSk7XG4gICAgcmV0dXJuIG5ldyBQcm9taXNlKChyZXNvbHZlLCByZWplY3QpID0+IHtcbiAgICAgICAgV2ViQXNzZW1ibHkuaW5zdGFudGlhdGUod2FzbU1vZHVsZSwgeyBlbnYgfSkudGhlbigoaW5zdGFuY2UpID0+IHtcbiAgICAgICAgICAgIGV4cG9ydHMgPSBpbnN0YW5jZS5leHBvcnRzO1xuICAgICAgICAgICAgaW5pdFRocmVhZExvY2FsU3RvcmFnZUFuZFN0YWNrT3RoZXJXb3JrZXJzKGV4cG9ydHMsIHRsc0FuZFN0YWNrRGF0YSk7XG4gICAgICAgICAgICAvLyBUT0RPKFBhcmFzKTogRXZlbnR1YWxseSBjYWxsIGBwcm9jZXNzV2FzbUV2ZW50c2AgaW5zdGVhZCBvZiBhIGN1c3RvbSBleHBvcnRlZCBmdW5jdGlvbi5cbiAgICAgICAgICAgIGV4cG9ydHMucnVuRnVuY3Rpb25Qb2ludGVyKGN0eFB0cik7XG4gICAgICAgICAgICByZXNvbHZlKCk7XG4gICAgICAgIH0sIHJlamVjdCk7XG4gICAgfSk7XG59KTtcbiJdLCJuYW1lcyI6W10sInNvdXJjZVJvb3QiOiIifQ==\n//# sourceURL=webpack-internal:///./async_worker.ts\n");

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = __webpack_modules__;
/******/ 	
/******/ 	// the startup function
/******/ 	__webpack_require__.x = () => {
/******/ 		// Load entry module and return exports
/******/ 		// This entry module depends on other loaded chunks and execution need to be delayed
/******/ 		var __webpack_exports__ = __webpack_require__.O(undefined, ["common_ts"], () => (__webpack_require__("./async_worker.ts")))
/******/ 		__webpack_exports__ = __webpack_require__.O(__webpack_exports__);
/******/ 		return __webpack_exports__;
/******/ 	};
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/chunk loaded */
/******/ 	(() => {
/******/ 		var deferred = [];
/******/ 		__webpack_require__.O = (result, chunkIds, fn, priority) => {
/******/ 			if(chunkIds) {
/******/ 				priority = priority || 0;
/******/ 				for(var i = deferred.length; i > 0 && deferred[i - 1][2] > priority; i--) deferred[i] = deferred[i - 1];
/******/ 				deferred[i] = [chunkIds, fn, priority];
/******/ 				return;
/******/ 			}
/******/ 			var notFulfilled = Infinity;
/******/ 			for (var i = 0; i < deferred.length; i++) {
/******/ 				var [chunkIds, fn, priority] = deferred[i];
/******/ 				var fulfilled = true;
/******/ 				for (var j = 0; j < chunkIds.length; j++) {
/******/ 					if ((priority & 1 === 0 || notFulfilled >= priority) && Object.keys(__webpack_require__.O).every((key) => (__webpack_require__.O[key](chunkIds[j])))) {
/******/ 						chunkIds.splice(j--, 1);
/******/ 					} else {
/******/ 						fulfilled = false;
/******/ 						if(priority < notFulfilled) notFulfilled = priority;
/******/ 					}
/******/ 				}
/******/ 				if(fulfilled) {
/******/ 					deferred.splice(i--, 1)
/******/ 					var r = fn();
/******/ 					if (r !== undefined) result = r;
/******/ 				}
/******/ 			}
/******/ 			return result;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/ensure chunk */
/******/ 	(() => {
/******/ 		__webpack_require__.f = {};
/******/ 		// This file contains only the entry chunk.
/******/ 		// The chunk loading function for additional chunks
/******/ 		__webpack_require__.e = (chunkId) => {
/******/ 			return Promise.all(Object.keys(__webpack_require__.f).reduce((promises, key) => {
/******/ 				__webpack_require__.f[key](chunkId, promises);
/******/ 				return promises;
/******/ 			}, []));
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/get javascript chunk filename */
/******/ 	(() => {
/******/ 		// This function allow to reference async chunks and sibling chunks for the entrypoint
/******/ 		__webpack_require__.u = (chunkId) => {
/******/ 			// return url for filenames based on template
/******/ 			return "" + chunkId + ".js";
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/global */
/******/ 	(() => {
/******/ 		__webpack_require__.g = (function() {
/******/ 			if (typeof globalThis === 'object') return globalThis;
/******/ 			try {
/******/ 				return this || new Function('return this')();
/******/ 			} catch (e) {
/******/ 				if (typeof window === 'object') return window;
/******/ 			}
/******/ 		})();
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/publicPath */
/******/ 	(() => {
/******/ 		var scriptUrl;
/******/ 		if (__webpack_require__.g.importScripts) scriptUrl = __webpack_require__.g.location + "";
/******/ 		var document = __webpack_require__.g.document;
/******/ 		if (!scriptUrl && document) {
/******/ 			if (document.currentScript)
/******/ 				scriptUrl = document.currentScript.src
/******/ 			if (!scriptUrl) {
/******/ 				var scripts = document.getElementsByTagName("script");
/******/ 				if(scripts.length) scriptUrl = scripts[scripts.length - 1].src
/******/ 			}
/******/ 		}
/******/ 		// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration
/******/ 		// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.
/******/ 		if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
/******/ 		scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
/******/ 		__webpack_require__.p = scriptUrl;
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/importScripts chunk loading */
/******/ 	(() => {
/******/ 		// no baseURI
/******/ 		
/******/ 		// object to store loaded chunks
/******/ 		// "1" means "already loaded"
/******/ 		var installedChunks = {
/******/ 			"async_worker_ts-_86481": 1
/******/ 		};
/******/ 		
/******/ 		// importScripts chunk loading
/******/ 		var installChunk = (data) => {
/******/ 			var [chunkIds, moreModules, runtime] = data;
/******/ 			for(var moduleId in moreModules) {
/******/ 				if(__webpack_require__.o(moreModules, moduleId)) {
/******/ 					__webpack_require__.m[moduleId] = moreModules[moduleId];
/******/ 				}
/******/ 			}
/******/ 			if(runtime) runtime(__webpack_require__);
/******/ 			while(chunkIds.length)
/******/ 				installedChunks[chunkIds.pop()] = 1;
/******/ 			parentChunkLoadingFunction(data);
/******/ 		};
/******/ 		__webpack_require__.f.i = (chunkId, promises) => {
/******/ 			// "1" is the signal for "already loaded"
/******/ 			if(!installedChunks[chunkId]) {
/******/ 				if(true) { // all chunks have JS
/******/ 					importScripts(__webpack_require__.p + __webpack_require__.u(chunkId));
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 		
/******/ 		var chunkLoadingGlobal = self["webpackChunkwrf"] = self["webpackChunkwrf"] || [];
/******/ 		var parentChunkLoadingFunction = chunkLoadingGlobal.push.bind(chunkLoadingGlobal);
/******/ 		chunkLoadingGlobal.push = installChunk;
/******/ 		
/******/ 		// no HMR
/******/ 		
/******/ 		// no HMR manifest
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/startup chunk dependencies */
/******/ 	(() => {
/******/ 		var next = __webpack_require__.x;
/******/ 		__webpack_require__.x = () => {
/******/ 			return __webpack_require__.e("common_ts").then(next);
/******/ 		};
/******/ 	})();
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// run startup
/******/ 	var __webpack_exports__ = __webpack_require__.x();
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});