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

/***/ "./task_worker.ts":
/*!************************!*\
  !*** ./task_worker.ts ***!
  \************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _common__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./common */ \"./common.ts\");\n/* harmony import */ var _rpc_types__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./rpc_types */ \"./rpc_types.ts\");\n/* harmony import */ var _zerde__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./zerde */ \"./zerde.ts\");\n// Copyright (c) 2021-present, Cruise LLC\n//\n// This source code is licensed under the Apache License, Version 2.0,\n// found in the LICENSE-APACHE file in the root directory of this source tree.\n// You may not use this file except in compliance with the License.\n\n\n\nconst _TASK_WORKER_INITIAL_RETURN_VALUE = -1;\nconst TASK_WORKER_ERROR_RETURN_VALUE = -2;\nconst rpc = new _common__WEBPACK_IMPORTED_MODULE_0__.Rpc(self);\nrpc.receive(_rpc_types__WEBPACK_IMPORTED_MODULE_1__.TaskWorkerEvent.Init, ({ taskWorkerSab, wasmMemory }) => {\n    const taskWorkerSabi32 = new Int32Array(taskWorkerSab);\n    // Number of async tasks that require the Javascript even loop to have control. If zero, we'll\n    // use Atomics.wait to wait for the next message, otherwise we'll use setTimeout to poll for new\n    // messages.\n    let asyncTasks = 0;\n    // HTTP streams. Start IDs with 1, since 0 signifies an error.\n    let nextStreamId = 1;\n    const streams = {};\n    // Send back an i32 return value, and wake up the original thread.\n    function sendi32ReturnValue(returnValPtr, returnValue) {\n        const memoryReturni32 = new Int32Array(wasmMemory.buffer, returnValPtr, 1);\n        if (memoryReturni32[0] === returnValue) {\n            throw new Error(\"Have to set the return value to something different than the initial value, otherwise Atomics.notify won't do anything\");\n        }\n        memoryReturni32[0] = returnValue;\n        Atomics.notify(memoryReturni32, 0);\n    }\n    // Make a new read request for a given stream. We do this even if the underlying application doesn't\n    // ask for it, so that we can return bytes in the fastest manner possible.\n    // TODO(JP): We might want to set a limit to how much we buffer ahead? Or make it configurable per stream?\n    function readDataIntoValuesBuffer(streamId) {\n        const stream = streams[streamId];\n        asyncTasks++;\n        stream.reader\n            .read()\n            .then((readResponse) => {\n            asyncTasks--;\n            if (readResponse.done) {\n                stream.done = true;\n            }\n            else {\n                stream.values.push(readResponse.value);\n                readDataIntoValuesBuffer(streamId);\n            }\n            handleHttpStreamRead(streamId);\n        })\n            .catch((error) => {\n            asyncTasks--;\n            // TODO(JP): Actually return the error to Rust at some point. For now we just print it.\n            console.error(\"fetch read error\", error);\n            stream.error = true;\n            handleHttpStreamRead(streamId);\n        });\n    }\n    // Check if we can supply a \"read\" call with data. There are two cases in which this can happen:\n    // * There is a new read call, and there is a sufficient amount of data to give it.\n    // * There is new data, and there is an existing read call to hand it to.\n    // In other cases we buffer the data or block the read call, and wait until we have enough of both.\n    function handleHttpStreamRead(streamId) {\n        const stream = streams[streamId];\n        if (!stream.currentTwMessage) {\n            // If there isn't a read call we can satisfy, bail.\n            return;\n        }\n        if (stream.error) {\n            sendi32ReturnValue(stream.currentTwMessage.bytesReadReturnValPtr, TASK_WORKER_ERROR_RETURN_VALUE);\n            stream.currentTwMessage = undefined;\n            return;\n        }\n        if (stream.values.length === 0) {\n            if (stream.done) {\n                // If there isn't more data, and we've reached the end of the stream, just return that we read 0 bytes.\n                sendi32ReturnValue(stream.currentTwMessage.bytesReadReturnValPtr, 0);\n                stream.currentTwMessage = undefined;\n            }\n            // If there is no more data but we're not done yet, just bail.\n            return;\n        }\n        // Read as many bytes as we can stuff in the buffer that was supplied to us from the read call.\n        let bytesRead = 0;\n        while (stream.values.length > 0 &&\n            bytesRead < stream.currentTwMessage.bufLen) {\n            const value = stream.values[0];\n            const remainingBytesToRead = stream.currentTwMessage.bufLen - bytesRead;\n            const bytesToReadFromValue = Math.min(value.byteLength, remainingBytesToRead);\n            const sourceBuffer = new Uint8Array(value.buffer, value.byteOffset, bytesToReadFromValue);\n            new Uint8Array(wasmMemory.buffer, stream.currentTwMessage.bufPtr + bytesRead, bytesToReadFromValue).set(sourceBuffer);\n            if (bytesToReadFromValue < value.byteLength) {\n                // If we weren't able to read the entire buffer, replace it with a buffer containing the rest.\n                stream.values[0] = new Uint8Array(value.buffer, value.byteOffset + bytesToReadFromValue, value.byteLength - bytesToReadFromValue);\n            }\n            else {\n                // If we read the whole buffer, remove it.\n                stream.values.shift();\n            }\n            bytesRead += bytesToReadFromValue;\n        }\n        // Return the number of bytes that we read.\n        sendi32ReturnValue(stream.currentTwMessage.bytesReadReturnValPtr, bytesRead);\n        stream.currentTwMessage = undefined;\n    }\n    // Parse a message, which is formatted using `ZerdeBuilder` in Rust, so we use `ZerdeParser` in Javascript\n    // to decode it.\n    function handleTwMessage(zerdeParser) {\n        const messageType = zerdeParser.parseU32();\n        if (messageType == 1) {\n            // http_stream_new\n            const streamIdReturnValPtr = zerdeParser.parseU32();\n            const url = zerdeParser.parseString();\n            const method = zerdeParser.parseString();\n            const body = zerdeParser.parseU8Slice();\n            const numberOfHeaders = zerdeParser.parseU32();\n            const headers = {};\n            for (let headerIndex = 0; headerIndex < numberOfHeaders; headerIndex++) {\n                headers[zerdeParser.parseString()] = zerdeParser.parseString();\n            }\n            asyncTasks++;\n            fetch(url, { method, body, headers })\n                .then((response) => {\n                asyncTasks--;\n                if (response.ok) {\n                    const streamId = nextStreamId++;\n                    streams[streamId] = {\n                        // An asynchronous reader, which returns \"chunks\"/\"values\" of data.\n                        // TODO(JP): Switch to \"byob\" when that's supported here; see\n                        // https://bugs.chromium.org/p/chromium/issues/detail?id=614302#c23\n                        reader: (0,_common__WEBPACK_IMPORTED_MODULE_0__.assertNotNull)(response.body).getReader(),\n                        // The buffered \"chunks\"/\"values\".\n                        values: [],\n                        // Whether we've read the whole stream into `values`.\n                        done: false,\n                        // Whether we encountered an error during reading.\n                        error: false,\n                        // The current read message to return data for, if any.\n                        currentTwMessage: undefined,\n                    };\n                    readDataIntoValuesBuffer(streamId);\n                    sendi32ReturnValue(streamIdReturnValPtr, streamId);\n                }\n                else {\n                    // TODO(JP): Actually return the status code to Rust at some point. For now you'll just\n                    // have to look at the Network tab of the browser's developer tools.\n                    sendi32ReturnValue(streamIdReturnValPtr, TASK_WORKER_ERROR_RETURN_VALUE);\n                }\n            })\n                .catch((error) => {\n                asyncTasks--;\n                // TODO(JP): Actually return the error to Rust at some point. For now we just print it.\n                console.error(\"fetch create error\", error);\n                sendi32ReturnValue(streamIdReturnValPtr, TASK_WORKER_ERROR_RETURN_VALUE);\n            });\n        }\n        else if (messageType == 2) {\n            // http_stream_read\n            const twMessage = {\n                bytesReadReturnValPtr: zerdeParser.parseU32(),\n                streamId: zerdeParser.parseU32(),\n                bufPtr: zerdeParser.parseU32(),\n                bufLen: zerdeParser.parseU32(),\n            };\n            if (streams[twMessage.streamId].currentTwMessage) {\n                // TODO(JP): Actually return the error to Rust at some point. For now we just print it.\n                console.error(\"Got multiple http_stream_read messages in a row\");\n                sendi32ReturnValue(twMessage.bytesReadReturnValPtr, TASK_WORKER_ERROR_RETURN_VALUE);\n                return;\n            }\n            streams[twMessage.streamId].currentTwMessage = twMessage;\n            handleHttpStreamRead(twMessage.streamId);\n        }\n    }\n    function process() {\n        // eslint-disable-next-line no-constant-condition\n        while (true) {\n            // Check if there are any messages. We do this without setting the Mutex, since\n            // assume that reads are always safe. Worse case we read an incorrect value, but\n            // a few lines down we read it again after having the Mutex.\n            if (Atomics.load(taskWorkerSabi32, _common__WEBPACK_IMPORTED_MODULE_0__.TW_SAB_MESSAGE_COUNT_PTR) > 0) {\n                (0,_common__WEBPACK_IMPORTED_MODULE_0__.mutexLock)(taskWorkerSabi32, _common__WEBPACK_IMPORTED_MODULE_0__.TW_SAB_MUTEX_PTR);\n                // Read the number of messages again now that we have the Mutex.\n                const numberOfMessages = taskWorkerSabi32[1];\n                // Handle all messages.\n                for (let messageIndex = 0; messageIndex < numberOfMessages; messageIndex++) {\n                    // Use unsigned numbers for the actual pointer, since they can be >2GB.\n                    const messagePtr = new Uint32Array(taskWorkerSab)[messageIndex + 2];\n                    handleTwMessage(new _zerde__WEBPACK_IMPORTED_MODULE_2__.ZerdeParser(wasmMemory, messagePtr));\n                }\n                // Reset the number of messages to 0.\n                taskWorkerSabi32[_common__WEBPACK_IMPORTED_MODULE_0__.TW_SAB_MESSAGE_COUNT_PTR] = 0;\n                (0,_common__WEBPACK_IMPORTED_MODULE_0__.mutexUnlock)(taskWorkerSabi32, _common__WEBPACK_IMPORTED_MODULE_0__.TW_SAB_MUTEX_PTR);\n            }\n            if (asyncTasks > 0) {\n                // We can't block if we have any async tasks currently running, since we need\n                // the Javascript event loop to be in control. So we queue up a new call to\n                // this function (which will be handled by the event loop!) and bail.\n                setTimeout(process, 1);\n                break;\n            }\n            else {\n                // Otherwise, we can safely block to wait for the next message.\n                Atomics.wait(taskWorkerSabi32, 1, 0);\n            }\n        }\n    }\n    // Queue up the first call to `process`. Don't call it directly, because it will likely immediately block,\n    // and it's nice to resolve the Promise associated with this \"init\" call (even though currently we don't\n    // actually use it).\n    setTimeout(process, 0);\n});\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi90YXNrX3dvcmtlci50cy5qcyIsIm1hcHBpbmdzIjoiOzs7O0FBQUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBIiwic291cmNlcyI6WyJ3ZWJwYWNrOi8vd3JmLy4vdGFza193b3JrZXIudHM/NThlZCJdLCJzb3VyY2VzQ29udGVudCI6WyIvLyBDb3B5cmlnaHQgKGMpIDIwMjEtcHJlc2VudCwgQ3J1aXNlIExMQ1xuLy9cbi8vIFRoaXMgc291cmNlIGNvZGUgaXMgbGljZW5zZWQgdW5kZXIgdGhlIEFwYWNoZSBMaWNlbnNlLCBWZXJzaW9uIDIuMCxcbi8vIGZvdW5kIGluIHRoZSBMSUNFTlNFLUFQQUNIRSBmaWxlIGluIHRoZSByb290IGRpcmVjdG9yeSBvZiB0aGlzIHNvdXJjZSB0cmVlLlxuLy8gWW91IG1heSBub3QgdXNlIHRoaXMgZmlsZSBleGNlcHQgaW4gY29tcGxpYW5jZSB3aXRoIHRoZSBMaWNlbnNlLlxuaW1wb3J0IHsgUnBjLCBUV19TQUJfTVVURVhfUFRSLCBUV19TQUJfTUVTU0FHRV9DT1VOVF9QVFIsIG11dGV4TG9jaywgbXV0ZXhVbmxvY2ssIGFzc2VydE5vdE51bGwsIH0gZnJvbSBcIi4vY29tbW9uXCI7XG5pbXBvcnQgeyBUYXNrV29ya2VyRXZlbnQgfSBmcm9tIFwiLi9ycGNfdHlwZXNcIjtcbmltcG9ydCB7IFplcmRlUGFyc2VyIH0gZnJvbSBcIi4vemVyZGVcIjtcbmNvbnN0IF9UQVNLX1dPUktFUl9JTklUSUFMX1JFVFVSTl9WQUxVRSA9IC0xO1xuY29uc3QgVEFTS19XT1JLRVJfRVJST1JfUkVUVVJOX1ZBTFVFID0gLTI7XG5jb25zdCBycGMgPSBuZXcgUnBjKHNlbGYpO1xucnBjLnJlY2VpdmUoVGFza1dvcmtlckV2ZW50LkluaXQsICh7IHRhc2tXb3JrZXJTYWIsIHdhc21NZW1vcnkgfSkgPT4ge1xuICAgIGNvbnN0IHRhc2tXb3JrZXJTYWJpMzIgPSBuZXcgSW50MzJBcnJheSh0YXNrV29ya2VyU2FiKTtcbiAgICAvLyBOdW1iZXIgb2YgYXN5bmMgdGFza3MgdGhhdCByZXF1aXJlIHRoZSBKYXZhc2NyaXB0IGV2ZW4gbG9vcCB0byBoYXZlIGNvbnRyb2wuIElmIHplcm8sIHdlJ2xsXG4gICAgLy8gdXNlIEF0b21pY3Mud2FpdCB0byB3YWl0IGZvciB0aGUgbmV4dCBtZXNzYWdlLCBvdGhlcndpc2Ugd2UnbGwgdXNlIHNldFRpbWVvdXQgdG8gcG9sbCBmb3IgbmV3XG4gICAgLy8gbWVzc2FnZXMuXG4gICAgbGV0IGFzeW5jVGFza3MgPSAwO1xuICAgIC8vIEhUVFAgc3RyZWFtcy4gU3RhcnQgSURzIHdpdGggMSwgc2luY2UgMCBzaWduaWZpZXMgYW4gZXJyb3IuXG4gICAgbGV0IG5leHRTdHJlYW1JZCA9IDE7XG4gICAgY29uc3Qgc3RyZWFtcyA9IHt9O1xuICAgIC8vIFNlbmQgYmFjayBhbiBpMzIgcmV0dXJuIHZhbHVlLCBhbmQgd2FrZSB1cCB0aGUgb3JpZ2luYWwgdGhyZWFkLlxuICAgIGZ1bmN0aW9uIHNlbmRpMzJSZXR1cm5WYWx1ZShyZXR1cm5WYWxQdHIsIHJldHVyblZhbHVlKSB7XG4gICAgICAgIGNvbnN0IG1lbW9yeVJldHVybmkzMiA9IG5ldyBJbnQzMkFycmF5KHdhc21NZW1vcnkuYnVmZmVyLCByZXR1cm5WYWxQdHIsIDEpO1xuICAgICAgICBpZiAobWVtb3J5UmV0dXJuaTMyWzBdID09PSByZXR1cm5WYWx1ZSkge1xuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKFwiSGF2ZSB0byBzZXQgdGhlIHJldHVybiB2YWx1ZSB0byBzb21ldGhpbmcgZGlmZmVyZW50IHRoYW4gdGhlIGluaXRpYWwgdmFsdWUsIG90aGVyd2lzZSBBdG9taWNzLm5vdGlmeSB3b24ndCBkbyBhbnl0aGluZ1wiKTtcbiAgICAgICAgfVxuICAgICAgICBtZW1vcnlSZXR1cm5pMzJbMF0gPSByZXR1cm5WYWx1ZTtcbiAgICAgICAgQXRvbWljcy5ub3RpZnkobWVtb3J5UmV0dXJuaTMyLCAwKTtcbiAgICB9XG4gICAgLy8gTWFrZSBhIG5ldyByZWFkIHJlcXVlc3QgZm9yIGEgZ2l2ZW4gc3RyZWFtLiBXZSBkbyB0aGlzIGV2ZW4gaWYgdGhlIHVuZGVybHlpbmcgYXBwbGljYXRpb24gZG9lc24ndFxuICAgIC8vIGFzayBmb3IgaXQsIHNvIHRoYXQgd2UgY2FuIHJldHVybiBieXRlcyBpbiB0aGUgZmFzdGVzdCBtYW5uZXIgcG9zc2libGUuXG4gICAgLy8gVE9ETyhKUCk6IFdlIG1pZ2h0IHdhbnQgdG8gc2V0IGEgbGltaXQgdG8gaG93IG11Y2ggd2UgYnVmZmVyIGFoZWFkPyBPciBtYWtlIGl0IGNvbmZpZ3VyYWJsZSBwZXIgc3RyZWFtP1xuICAgIGZ1bmN0aW9uIHJlYWREYXRhSW50b1ZhbHVlc0J1ZmZlcihzdHJlYW1JZCkge1xuICAgICAgICBjb25zdCBzdHJlYW0gPSBzdHJlYW1zW3N0cmVhbUlkXTtcbiAgICAgICAgYXN5bmNUYXNrcysrO1xuICAgICAgICBzdHJlYW0ucmVhZGVyXG4gICAgICAgICAgICAucmVhZCgpXG4gICAgICAgICAgICAudGhlbigocmVhZFJlc3BvbnNlKSA9PiB7XG4gICAgICAgICAgICBhc3luY1Rhc2tzLS07XG4gICAgICAgICAgICBpZiAocmVhZFJlc3BvbnNlLmRvbmUpIHtcbiAgICAgICAgICAgICAgICBzdHJlYW0uZG9uZSA9IHRydWU7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBlbHNlIHtcbiAgICAgICAgICAgICAgICBzdHJlYW0udmFsdWVzLnB1c2gocmVhZFJlc3BvbnNlLnZhbHVlKTtcbiAgICAgICAgICAgICAgICByZWFkRGF0YUludG9WYWx1ZXNCdWZmZXIoc3RyZWFtSWQpO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgaGFuZGxlSHR0cFN0cmVhbVJlYWQoc3RyZWFtSWQpO1xuICAgICAgICB9KVxuICAgICAgICAgICAgLmNhdGNoKChlcnJvcikgPT4ge1xuICAgICAgICAgICAgYXN5bmNUYXNrcy0tO1xuICAgICAgICAgICAgLy8gVE9ETyhKUCk6IEFjdHVhbGx5IHJldHVybiB0aGUgZXJyb3IgdG8gUnVzdCBhdCBzb21lIHBvaW50LiBGb3Igbm93IHdlIGp1c3QgcHJpbnQgaXQuXG4gICAgICAgICAgICBjb25zb2xlLmVycm9yKFwiZmV0Y2ggcmVhZCBlcnJvclwiLCBlcnJvcik7XG4gICAgICAgICAgICBzdHJlYW0uZXJyb3IgPSB0cnVlO1xuICAgICAgICAgICAgaGFuZGxlSHR0cFN0cmVhbVJlYWQoc3RyZWFtSWQpO1xuICAgICAgICB9KTtcbiAgICB9XG4gICAgLy8gQ2hlY2sgaWYgd2UgY2FuIHN1cHBseSBhIFwicmVhZFwiIGNhbGwgd2l0aCBkYXRhLiBUaGVyZSBhcmUgdHdvIGNhc2VzIGluIHdoaWNoIHRoaXMgY2FuIGhhcHBlbjpcbiAgICAvLyAqIFRoZXJlIGlzIGEgbmV3IHJlYWQgY2FsbCwgYW5kIHRoZXJlIGlzIGEgc3VmZmljaWVudCBhbW91bnQgb2YgZGF0YSB0byBnaXZlIGl0LlxuICAgIC8vICogVGhlcmUgaXMgbmV3IGRhdGEsIGFuZCB0aGVyZSBpcyBhbiBleGlzdGluZyByZWFkIGNhbGwgdG8gaGFuZCBpdCB0by5cbiAgICAvLyBJbiBvdGhlciBjYXNlcyB3ZSBidWZmZXIgdGhlIGRhdGEgb3IgYmxvY2sgdGhlIHJlYWQgY2FsbCwgYW5kIHdhaXQgdW50aWwgd2UgaGF2ZSBlbm91Z2ggb2YgYm90aC5cbiAgICBmdW5jdGlvbiBoYW5kbGVIdHRwU3RyZWFtUmVhZChzdHJlYW1JZCkge1xuICAgICAgICBjb25zdCBzdHJlYW0gPSBzdHJlYW1zW3N0cmVhbUlkXTtcbiAgICAgICAgaWYgKCFzdHJlYW0uY3VycmVudFR3TWVzc2FnZSkge1xuICAgICAgICAgICAgLy8gSWYgdGhlcmUgaXNuJ3QgYSByZWFkIGNhbGwgd2UgY2FuIHNhdGlzZnksIGJhaWwuXG4gICAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgaWYgKHN0cmVhbS5lcnJvcikge1xuICAgICAgICAgICAgc2VuZGkzMlJldHVyblZhbHVlKHN0cmVhbS5jdXJyZW50VHdNZXNzYWdlLmJ5dGVzUmVhZFJldHVyblZhbFB0ciwgVEFTS19XT1JLRVJfRVJST1JfUkVUVVJOX1ZBTFVFKTtcbiAgICAgICAgICAgIHN0cmVhbS5jdXJyZW50VHdNZXNzYWdlID0gdW5kZWZpbmVkO1xuICAgICAgICAgICAgcmV0dXJuO1xuICAgICAgICB9XG4gICAgICAgIGlmIChzdHJlYW0udmFsdWVzLmxlbmd0aCA9PT0gMCkge1xuICAgICAgICAgICAgaWYgKHN0cmVhbS5kb25lKSB7XG4gICAgICAgICAgICAgICAgLy8gSWYgdGhlcmUgaXNuJ3QgbW9yZSBkYXRhLCBhbmQgd2UndmUgcmVhY2hlZCB0aGUgZW5kIG9mIHRoZSBzdHJlYW0sIGp1c3QgcmV0dXJuIHRoYXQgd2UgcmVhZCAwIGJ5dGVzLlxuICAgICAgICAgICAgICAgIHNlbmRpMzJSZXR1cm5WYWx1ZShzdHJlYW0uY3VycmVudFR3TWVzc2FnZS5ieXRlc1JlYWRSZXR1cm5WYWxQdHIsIDApO1xuICAgICAgICAgICAgICAgIHN0cmVhbS5jdXJyZW50VHdNZXNzYWdlID0gdW5kZWZpbmVkO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgLy8gSWYgdGhlcmUgaXMgbm8gbW9yZSBkYXRhIGJ1dCB3ZSdyZSBub3QgZG9uZSB5ZXQsIGp1c3QgYmFpbC5cbiAgICAgICAgICAgIHJldHVybjtcbiAgICAgICAgfVxuICAgICAgICAvLyBSZWFkIGFzIG1hbnkgYnl0ZXMgYXMgd2UgY2FuIHN0dWZmIGluIHRoZSBidWZmZXIgdGhhdCB3YXMgc3VwcGxpZWQgdG8gdXMgZnJvbSB0aGUgcmVhZCBjYWxsLlxuICAgICAgICBsZXQgYnl0ZXNSZWFkID0gMDtcbiAgICAgICAgd2hpbGUgKHN0cmVhbS52YWx1ZXMubGVuZ3RoID4gMCAmJlxuICAgICAgICAgICAgYnl0ZXNSZWFkIDwgc3RyZWFtLmN1cnJlbnRUd01lc3NhZ2UuYnVmTGVuKSB7XG4gICAgICAgICAgICBjb25zdCB2YWx1ZSA9IHN0cmVhbS52YWx1ZXNbMF07XG4gICAgICAgICAgICBjb25zdCByZW1haW5pbmdCeXRlc1RvUmVhZCA9IHN0cmVhbS5jdXJyZW50VHdNZXNzYWdlLmJ1ZkxlbiAtIGJ5dGVzUmVhZDtcbiAgICAgICAgICAgIGNvbnN0IGJ5dGVzVG9SZWFkRnJvbVZhbHVlID0gTWF0aC5taW4odmFsdWUuYnl0ZUxlbmd0aCwgcmVtYWluaW5nQnl0ZXNUb1JlYWQpO1xuICAgICAgICAgICAgY29uc3Qgc291cmNlQnVmZmVyID0gbmV3IFVpbnQ4QXJyYXkodmFsdWUuYnVmZmVyLCB2YWx1ZS5ieXRlT2Zmc2V0LCBieXRlc1RvUmVhZEZyb21WYWx1ZSk7XG4gICAgICAgICAgICBuZXcgVWludDhBcnJheSh3YXNtTWVtb3J5LmJ1ZmZlciwgc3RyZWFtLmN1cnJlbnRUd01lc3NhZ2UuYnVmUHRyICsgYnl0ZXNSZWFkLCBieXRlc1RvUmVhZEZyb21WYWx1ZSkuc2V0KHNvdXJjZUJ1ZmZlcik7XG4gICAgICAgICAgICBpZiAoYnl0ZXNUb1JlYWRGcm9tVmFsdWUgPCB2YWx1ZS5ieXRlTGVuZ3RoKSB7XG4gICAgICAgICAgICAgICAgLy8gSWYgd2Ugd2VyZW4ndCBhYmxlIHRvIHJlYWQgdGhlIGVudGlyZSBidWZmZXIsIHJlcGxhY2UgaXQgd2l0aCBhIGJ1ZmZlciBjb250YWluaW5nIHRoZSByZXN0LlxuICAgICAgICAgICAgICAgIHN0cmVhbS52YWx1ZXNbMF0gPSBuZXcgVWludDhBcnJheSh2YWx1ZS5idWZmZXIsIHZhbHVlLmJ5dGVPZmZzZXQgKyBieXRlc1RvUmVhZEZyb21WYWx1ZSwgdmFsdWUuYnl0ZUxlbmd0aCAtIGJ5dGVzVG9SZWFkRnJvbVZhbHVlKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGVsc2Uge1xuICAgICAgICAgICAgICAgIC8vIElmIHdlIHJlYWQgdGhlIHdob2xlIGJ1ZmZlciwgcmVtb3ZlIGl0LlxuICAgICAgICAgICAgICAgIHN0cmVhbS52YWx1ZXMuc2hpZnQoKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGJ5dGVzUmVhZCArPSBieXRlc1RvUmVhZEZyb21WYWx1ZTtcbiAgICAgICAgfVxuICAgICAgICAvLyBSZXR1cm4gdGhlIG51bWJlciBvZiBieXRlcyB0aGF0IHdlIHJlYWQuXG4gICAgICAgIHNlbmRpMzJSZXR1cm5WYWx1ZShzdHJlYW0uY3VycmVudFR3TWVzc2FnZS5ieXRlc1JlYWRSZXR1cm5WYWxQdHIsIGJ5dGVzUmVhZCk7XG4gICAgICAgIHN0cmVhbS5jdXJyZW50VHdNZXNzYWdlID0gdW5kZWZpbmVkO1xuICAgIH1cbiAgICAvLyBQYXJzZSBhIG1lc3NhZ2UsIHdoaWNoIGlzIGZvcm1hdHRlZCB1c2luZyBgWmVyZGVCdWlsZGVyYCBpbiBSdXN0LCBzbyB3ZSB1c2UgYFplcmRlUGFyc2VyYCBpbiBKYXZhc2NyaXB0XG4gICAgLy8gdG8gZGVjb2RlIGl0LlxuICAgIGZ1bmN0aW9uIGhhbmRsZVR3TWVzc2FnZSh6ZXJkZVBhcnNlcikge1xuICAgICAgICBjb25zdCBtZXNzYWdlVHlwZSA9IHplcmRlUGFyc2VyLnBhcnNlVTMyKCk7XG4gICAgICAgIGlmIChtZXNzYWdlVHlwZSA9PSAxKSB7XG4gICAgICAgICAgICAvLyBodHRwX3N0cmVhbV9uZXdcbiAgICAgICAgICAgIGNvbnN0IHN0cmVhbUlkUmV0dXJuVmFsUHRyID0gemVyZGVQYXJzZXIucGFyc2VVMzIoKTtcbiAgICAgICAgICAgIGNvbnN0IHVybCA9IHplcmRlUGFyc2VyLnBhcnNlU3RyaW5nKCk7XG4gICAgICAgICAgICBjb25zdCBtZXRob2QgPSB6ZXJkZVBhcnNlci5wYXJzZVN0cmluZygpO1xuICAgICAgICAgICAgY29uc3QgYm9keSA9IHplcmRlUGFyc2VyLnBhcnNlVThTbGljZSgpO1xuICAgICAgICAgICAgY29uc3QgbnVtYmVyT2ZIZWFkZXJzID0gemVyZGVQYXJzZXIucGFyc2VVMzIoKTtcbiAgICAgICAgICAgIGNvbnN0IGhlYWRlcnMgPSB7fTtcbiAgICAgICAgICAgIGZvciAobGV0IGhlYWRlckluZGV4ID0gMDsgaGVhZGVySW5kZXggPCBudW1iZXJPZkhlYWRlcnM7IGhlYWRlckluZGV4KyspIHtcbiAgICAgICAgICAgICAgICBoZWFkZXJzW3plcmRlUGFyc2VyLnBhcnNlU3RyaW5nKCldID0gemVyZGVQYXJzZXIucGFyc2VTdHJpbmcoKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGFzeW5jVGFza3MrKztcbiAgICAgICAgICAgIGZldGNoKHVybCwgeyBtZXRob2QsIGJvZHksIGhlYWRlcnMgfSlcbiAgICAgICAgICAgICAgICAudGhlbigocmVzcG9uc2UpID0+IHtcbiAgICAgICAgICAgICAgICBhc3luY1Rhc2tzLS07XG4gICAgICAgICAgICAgICAgaWYgKHJlc3BvbnNlLm9rKSB7XG4gICAgICAgICAgICAgICAgICAgIGNvbnN0IHN0cmVhbUlkID0gbmV4dFN0cmVhbUlkKys7XG4gICAgICAgICAgICAgICAgICAgIHN0cmVhbXNbc3RyZWFtSWRdID0ge1xuICAgICAgICAgICAgICAgICAgICAgICAgLy8gQW4gYXN5bmNocm9ub3VzIHJlYWRlciwgd2hpY2ggcmV0dXJucyBcImNodW5rc1wiL1widmFsdWVzXCIgb2YgZGF0YS5cbiAgICAgICAgICAgICAgICAgICAgICAgIC8vIFRPRE8oSlApOiBTd2l0Y2ggdG8gXCJieW9iXCIgd2hlbiB0aGF0J3Mgc3VwcG9ydGVkIGhlcmU7IHNlZVxuICAgICAgICAgICAgICAgICAgICAgICAgLy8gaHR0cHM6Ly9idWdzLmNocm9taXVtLm9yZy9wL2Nocm9taXVtL2lzc3Vlcy9kZXRhaWw/aWQ9NjE0MzAyI2MyM1xuICAgICAgICAgICAgICAgICAgICAgICAgcmVhZGVyOiBhc3NlcnROb3ROdWxsKHJlc3BvbnNlLmJvZHkpLmdldFJlYWRlcigpLFxuICAgICAgICAgICAgICAgICAgICAgICAgLy8gVGhlIGJ1ZmZlcmVkIFwiY2h1bmtzXCIvXCJ2YWx1ZXNcIi5cbiAgICAgICAgICAgICAgICAgICAgICAgIHZhbHVlczogW10sXG4gICAgICAgICAgICAgICAgICAgICAgICAvLyBXaGV0aGVyIHdlJ3ZlIHJlYWQgdGhlIHdob2xlIHN0cmVhbSBpbnRvIGB2YWx1ZXNgLlxuICAgICAgICAgICAgICAgICAgICAgICAgZG9uZTogZmFsc2UsXG4gICAgICAgICAgICAgICAgICAgICAgICAvLyBXaGV0aGVyIHdlIGVuY291bnRlcmVkIGFuIGVycm9yIGR1cmluZyByZWFkaW5nLlxuICAgICAgICAgICAgICAgICAgICAgICAgZXJyb3I6IGZhbHNlLFxuICAgICAgICAgICAgICAgICAgICAgICAgLy8gVGhlIGN1cnJlbnQgcmVhZCBtZXNzYWdlIHRvIHJldHVybiBkYXRhIGZvciwgaWYgYW55LlxuICAgICAgICAgICAgICAgICAgICAgICAgY3VycmVudFR3TWVzc2FnZTogdW5kZWZpbmVkLFxuICAgICAgICAgICAgICAgICAgICB9O1xuICAgICAgICAgICAgICAgICAgICByZWFkRGF0YUludG9WYWx1ZXNCdWZmZXIoc3RyZWFtSWQpO1xuICAgICAgICAgICAgICAgICAgICBzZW5kaTMyUmV0dXJuVmFsdWUoc3RyZWFtSWRSZXR1cm5WYWxQdHIsIHN0cmVhbUlkKTtcbiAgICAgICAgICAgICAgICB9XG4gICAgICAgICAgICAgICAgZWxzZSB7XG4gICAgICAgICAgICAgICAgICAgIC8vIFRPRE8oSlApOiBBY3R1YWxseSByZXR1cm4gdGhlIHN0YXR1cyBjb2RlIHRvIFJ1c3QgYXQgc29tZSBwb2ludC4gRm9yIG5vdyB5b3UnbGwganVzdFxuICAgICAgICAgICAgICAgICAgICAvLyBoYXZlIHRvIGxvb2sgYXQgdGhlIE5ldHdvcmsgdGFiIG9mIHRoZSBicm93c2VyJ3MgZGV2ZWxvcGVyIHRvb2xzLlxuICAgICAgICAgICAgICAgICAgICBzZW5kaTMyUmV0dXJuVmFsdWUoc3RyZWFtSWRSZXR1cm5WYWxQdHIsIFRBU0tfV09SS0VSX0VSUk9SX1JFVFVSTl9WQUxVRSk7XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgfSlcbiAgICAgICAgICAgICAgICAuY2F0Y2goKGVycm9yKSA9PiB7XG4gICAgICAgICAgICAgICAgYXN5bmNUYXNrcy0tO1xuICAgICAgICAgICAgICAgIC8vIFRPRE8oSlApOiBBY3R1YWxseSByZXR1cm4gdGhlIGVycm9yIHRvIFJ1c3QgYXQgc29tZSBwb2ludC4gRm9yIG5vdyB3ZSBqdXN0IHByaW50IGl0LlxuICAgICAgICAgICAgICAgIGNvbnNvbGUuZXJyb3IoXCJmZXRjaCBjcmVhdGUgZXJyb3JcIiwgZXJyb3IpO1xuICAgICAgICAgICAgICAgIHNlbmRpMzJSZXR1cm5WYWx1ZShzdHJlYW1JZFJldHVyblZhbFB0ciwgVEFTS19XT1JLRVJfRVJST1JfUkVUVVJOX1ZBTFVFKTtcbiAgICAgICAgICAgIH0pO1xuICAgICAgICB9XG4gICAgICAgIGVsc2UgaWYgKG1lc3NhZ2VUeXBlID09IDIpIHtcbiAgICAgICAgICAgIC8vIGh0dHBfc3RyZWFtX3JlYWRcbiAgICAgICAgICAgIGNvbnN0IHR3TWVzc2FnZSA9IHtcbiAgICAgICAgICAgICAgICBieXRlc1JlYWRSZXR1cm5WYWxQdHI6IHplcmRlUGFyc2VyLnBhcnNlVTMyKCksXG4gICAgICAgICAgICAgICAgc3RyZWFtSWQ6IHplcmRlUGFyc2VyLnBhcnNlVTMyKCksXG4gICAgICAgICAgICAgICAgYnVmUHRyOiB6ZXJkZVBhcnNlci5wYXJzZVUzMigpLFxuICAgICAgICAgICAgICAgIGJ1ZkxlbjogemVyZGVQYXJzZXIucGFyc2VVMzIoKSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgICBpZiAoc3RyZWFtc1t0d01lc3NhZ2Uuc3RyZWFtSWRdLmN1cnJlbnRUd01lc3NhZ2UpIHtcbiAgICAgICAgICAgICAgICAvLyBUT0RPKEpQKTogQWN0dWFsbHkgcmV0dXJuIHRoZSBlcnJvciB0byBSdXN0IGF0IHNvbWUgcG9pbnQuIEZvciBub3cgd2UganVzdCBwcmludCBpdC5cbiAgICAgICAgICAgICAgICBjb25zb2xlLmVycm9yKFwiR290IG11bHRpcGxlIGh0dHBfc3RyZWFtX3JlYWQgbWVzc2FnZXMgaW4gYSByb3dcIik7XG4gICAgICAgICAgICAgICAgc2VuZGkzMlJldHVyblZhbHVlKHR3TWVzc2FnZS5ieXRlc1JlYWRSZXR1cm5WYWxQdHIsIFRBU0tfV09SS0VSX0VSUk9SX1JFVFVSTl9WQUxVRSk7XG4gICAgICAgICAgICAgICAgcmV0dXJuO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgc3RyZWFtc1t0d01lc3NhZ2Uuc3RyZWFtSWRdLmN1cnJlbnRUd01lc3NhZ2UgPSB0d01lc3NhZ2U7XG4gICAgICAgICAgICBoYW5kbGVIdHRwU3RyZWFtUmVhZCh0d01lc3NhZ2Uuc3RyZWFtSWQpO1xuICAgICAgICB9XG4gICAgfVxuICAgIGZ1bmN0aW9uIHByb2Nlc3MoKSB7XG4gICAgICAgIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZSBuby1jb25zdGFudC1jb25kaXRpb25cbiAgICAgICAgd2hpbGUgKHRydWUpIHtcbiAgICAgICAgICAgIC8vIENoZWNrIGlmIHRoZXJlIGFyZSBhbnkgbWVzc2FnZXMuIFdlIGRvIHRoaXMgd2l0aG91dCBzZXR0aW5nIHRoZSBNdXRleCwgc2luY2VcbiAgICAgICAgICAgIC8vIGFzc3VtZSB0aGF0IHJlYWRzIGFyZSBhbHdheXMgc2FmZS4gV29yc2UgY2FzZSB3ZSByZWFkIGFuIGluY29ycmVjdCB2YWx1ZSwgYnV0XG4gICAgICAgICAgICAvLyBhIGZldyBsaW5lcyBkb3duIHdlIHJlYWQgaXQgYWdhaW4gYWZ0ZXIgaGF2aW5nIHRoZSBNdXRleC5cbiAgICAgICAgICAgIGlmIChBdG9taWNzLmxvYWQodGFza1dvcmtlclNhYmkzMiwgVFdfU0FCX01FU1NBR0VfQ09VTlRfUFRSKSA+IDApIHtcbiAgICAgICAgICAgICAgICBtdXRleExvY2sodGFza1dvcmtlclNhYmkzMiwgVFdfU0FCX01VVEVYX1BUUik7XG4gICAgICAgICAgICAgICAgLy8gUmVhZCB0aGUgbnVtYmVyIG9mIG1lc3NhZ2VzIGFnYWluIG5vdyB0aGF0IHdlIGhhdmUgdGhlIE11dGV4LlxuICAgICAgICAgICAgICAgIGNvbnN0IG51bWJlck9mTWVzc2FnZXMgPSB0YXNrV29ya2VyU2FiaTMyWzFdO1xuICAgICAgICAgICAgICAgIC8vIEhhbmRsZSBhbGwgbWVzc2FnZXMuXG4gICAgICAgICAgICAgICAgZm9yIChsZXQgbWVzc2FnZUluZGV4ID0gMDsgbWVzc2FnZUluZGV4IDwgbnVtYmVyT2ZNZXNzYWdlczsgbWVzc2FnZUluZGV4KyspIHtcbiAgICAgICAgICAgICAgICAgICAgLy8gVXNlIHVuc2lnbmVkIG51bWJlcnMgZm9yIHRoZSBhY3R1YWwgcG9pbnRlciwgc2luY2UgdGhleSBjYW4gYmUgPjJHQi5cbiAgICAgICAgICAgICAgICAgICAgY29uc3QgbWVzc2FnZVB0ciA9IG5ldyBVaW50MzJBcnJheSh0YXNrV29ya2VyU2FiKVttZXNzYWdlSW5kZXggKyAyXTtcbiAgICAgICAgICAgICAgICAgICAgaGFuZGxlVHdNZXNzYWdlKG5ldyBaZXJkZVBhcnNlcih3YXNtTWVtb3J5LCBtZXNzYWdlUHRyKSk7XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgIC8vIFJlc2V0IHRoZSBudW1iZXIgb2YgbWVzc2FnZXMgdG8gMC5cbiAgICAgICAgICAgICAgICB0YXNrV29ya2VyU2FiaTMyW1RXX1NBQl9NRVNTQUdFX0NPVU5UX1BUUl0gPSAwO1xuICAgICAgICAgICAgICAgIG11dGV4VW5sb2NrKHRhc2tXb3JrZXJTYWJpMzIsIFRXX1NBQl9NVVRFWF9QVFIpO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgaWYgKGFzeW5jVGFza3MgPiAwKSB7XG4gICAgICAgICAgICAgICAgLy8gV2UgY2FuJ3QgYmxvY2sgaWYgd2UgaGF2ZSBhbnkgYXN5bmMgdGFza3MgY3VycmVudGx5IHJ1bm5pbmcsIHNpbmNlIHdlIG5lZWRcbiAgICAgICAgICAgICAgICAvLyB0aGUgSmF2YXNjcmlwdCBldmVudCBsb29wIHRvIGJlIGluIGNvbnRyb2wuIFNvIHdlIHF1ZXVlIHVwIGEgbmV3IGNhbGwgdG9cbiAgICAgICAgICAgICAgICAvLyB0aGlzIGZ1bmN0aW9uICh3aGljaCB3aWxsIGJlIGhhbmRsZWQgYnkgdGhlIGV2ZW50IGxvb3AhKSBhbmQgYmFpbC5cbiAgICAgICAgICAgICAgICBzZXRUaW1lb3V0KHByb2Nlc3MsIDEpO1xuICAgICAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgZWxzZSB7XG4gICAgICAgICAgICAgICAgLy8gT3RoZXJ3aXNlLCB3ZSBjYW4gc2FmZWx5IGJsb2NrIHRvIHdhaXQgZm9yIHRoZSBuZXh0IG1lc3NhZ2UuXG4gICAgICAgICAgICAgICAgQXRvbWljcy53YWl0KHRhc2tXb3JrZXJTYWJpMzIsIDEsIDApO1xuICAgICAgICAgICAgfVxuICAgICAgICB9XG4gICAgfVxuICAgIC8vIFF1ZXVlIHVwIHRoZSBmaXJzdCBjYWxsIHRvIGBwcm9jZXNzYC4gRG9uJ3QgY2FsbCBpdCBkaXJlY3RseSwgYmVjYXVzZSBpdCB3aWxsIGxpa2VseSBpbW1lZGlhdGVseSBibG9jayxcbiAgICAvLyBhbmQgaXQncyBuaWNlIHRvIHJlc29sdmUgdGhlIFByb21pc2UgYXNzb2NpYXRlZCB3aXRoIHRoaXMgXCJpbml0XCIgY2FsbCAoZXZlbiB0aG91Z2ggY3VycmVudGx5IHdlIGRvbid0XG4gICAgLy8gYWN0dWFsbHkgdXNlIGl0KS5cbiAgICBzZXRUaW1lb3V0KHByb2Nlc3MsIDApO1xufSk7XG4iXSwibmFtZXMiOltdLCJzb3VyY2VSb290IjoiIn0=\n//# sourceURL=webpack-internal:///./task_worker.ts\n");

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
/******/ 		var __webpack_exports__ = __webpack_require__.O(undefined, ["rpc_types_ts-wrf_buffer_ts"], () => (__webpack_require__("./task_worker.ts")))
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
/******/ 			"task_worker_ts": 1
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
/******/ 			return __webpack_require__.e("rpc_types_ts-wrf_buffer_ts").then(next);
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