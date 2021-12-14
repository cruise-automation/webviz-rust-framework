// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { cursorMap } from "./cursor_map";
import {
  Rpc,
  initTaskWorkerSab,
  getWasmEnv,
  copyUint8ArrayToRustBuffer,
} from "./common";
import { RpcEvent } from "./make_rpc_event";
import {
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "./make_textarea";
import {
  FileHandle,
  ShaderAttributes,
  Texture,
  Uniform,
  WasmExports,
  CallJSData,
  BufferData,
  UserWorkerInitReturnValue,
  PostMessageTypedArray,
  WorkerEvent,
  UserWorkerEvent,
  AsyncWorkerEvent,
  TaskWorkerEvent,
} from "./types";
import { ZerdeParser } from "./zerde";
import { ZerdeEventloopEvents } from "./zerde_eventloop_events";
import { packKeyModifier } from "./zerde_keyboard_handlers";

const rpc = new Rpc(self);

const isFirefox =
  self.navigator.userAgent.toLowerCase().indexOf("firefox") > -1;
// var is_add_to_homescreen_safari = is_mobile_safari && navigator.standalone;
//var is_oculus_browser = navigator.userAgent.indexOf('OculusBrowser') > -1;

type Timer = { id: number; repeats: number; sysId: number };

export type Dependency = { vecPtr: number; name: string; vecLen: number };

type Resource = { name: string; buffer: ArrayBuffer };

export type Finger = {
  x: number;
  y: number;
  digit: number;
  time: number;
  modifiers: number;
  touch: boolean;
};

export type FingerScroll = Finger & {
  scrollX: number;
  scrollY: number;
  isWheel: boolean;
};

export class WasmApp {
  memory: WebAssembly.Memory;
  exports: WasmExports;
  canvas: OffscreenCanvas;
  module: WebAssembly.Module;
  canFullscreen: boolean;
  baseUri: string;
  shaders: {
    geomAttribs: ReturnType<WasmApp["getAttribLocations"]>;
    instAttribs: ReturnType<WasmApp["getAttribLocations"]>;
    passUniforms: ReturnType<WasmApp["getUniformLocations"]>;
    viewUniforms: ReturnType<WasmApp["getUniformLocations"]>;
    drawUniforms: ReturnType<WasmApp["getUniformLocations"]>;
    userUniforms: ReturnType<WasmApp["getUniformLocations"]>;
    textureSlots: ReturnType<WasmApp["getUniformLocations"]>;
    instanceSlots: number;
    program: WebGLProgram;
    ash: ShaderAttributes;
  }[];
  indexBuffers: { glBuf: WebGLBuffer; length: number }[];
  arrayBuffers: { glBuf: WebGLBuffer; length: number }[];
  timers: Timer[];
  vaos: {
    glVao: WebGLVertexArrayObjectOES;
    geomIbId: number;
    geomVbId: number;
    instVbId: number;
  }[];
  textures: Texture[];
  framebuffers: unknown[];
  resources: Promise<Resource>[];
  reqAnimFrameId: number;
  websockets: unknown;
  fileHandles: FileHandle[];
  zerdeEventloopEvents: ZerdeEventloopEvents;
  appPtr: BigInt;
  doWasmBlock: boolean;
  width: number;
  height: number;
  dpiFactor: number;
  xrCanPresent: boolean;
  xrIsPresenting: boolean;
  zerdeParser: ZerdeParser;
  basef32: Float32Array;
  baseu32: Uint32Array;
  basef64: Float64Array;
  baseu64: BigUint64Array;
  sendFnTable: ((self: this) => void | boolean)[];
  gl: WebGLRenderingContext;
  // eslint-disable-next-line camelcase
  OESStandardDerivatives: OES_standard_derivatives;
  // eslint-disable-next-line camelcase
  OESVertexArrayObject: OES_vertex_array_object;
  // eslint-disable-next-line camelcase
  OESElementIndexUint: OES_element_index_uint;
  // eslint-disable-next-line camelcase
  ANGLEInstancedArrays: ANGLE_instanced_arrays;
  inAnimationFrame: boolean;
  isMainCanvas: boolean;
  targetWidth: number;
  targetHeight: number;
  colorTargets: number;
  clearFlags: number;
  clearR: number;
  clearG: number;
  clearB: number;
  clearA: number;
  clearDepth: number;
  uniformFnTable: Record<
    string,
    (self: this, loc: WebGLUniformLocation, off: number) => void
  >;
  callRustNewCallbackId: number;
  callRustPendingCallbacks: Record<number, (arg0: any) => void>;

  constructor({
    offscreenCanvas,
    webasm,
    memory,
    canFullscreen,
    baseUri,
    fileHandles,
    taskWorkerSab,
  }: {
    offscreenCanvas: OffscreenCanvas;
    webasm: WebAssembly.WebAssemblyInstantiatedSource;
    memory: WebAssembly.Memory;
    canFullscreen: boolean;
    baseUri: string;
    fileHandles: FileHandle[];
    taskWorkerSab: SharedArrayBuffer;
  }) {
    this.canvas = offscreenCanvas;
    this.module = webasm.module;
    this.exports = webasm.instance.exports as WasmExports;
    this.memory = memory;
    this.canFullscreen = canFullscreen;
    this.baseUri = baseUri;

    // local webgl resources
    this.shaders = [];
    this.indexBuffers = [];
    this.arrayBuffers = [];
    this.timers = [];
    this.vaos = [];
    this.textures = [];
    this.framebuffers = [];
    this.resources = [];
    this.reqAnimFrameId = 0;
    this.websockets = {};
    this.fileHandles = fileHandles;

    this.callRustNewCallbackId = 0;
    this.callRustPendingCallbacks = {};

    this.initWebglContext();
    // this.run_async_webxr_check();
    this.bindMouseAndTouch();
    this.bindKeyboard();

    this.appPtr = this.exports.createWasmApp();

    rpc.receive(WorkerEvent.WindowFocus, () => {
      this.zerdeEventloopEvents.windowFocus(true);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowBlur, () => {
      this.zerdeEventloopEvents.windowFocus(false);
      this.doWasmIo();
    });

    const callRust = ({
      name,
      params,
    }: {
      name: string;
      params: (string | PostMessageTypedArray | Uint8Array)[];
    }): Promise<(string | BufferData)[]> => {
      const callbackId = this.callRustNewCallbackId++;
      const promise = new Promise<(string | BufferData)[]>(
        (resolve, _reject) => {
          this.callRustPendingCallbacks[callbackId] = (data) => {
            // TODO(Dmitry): implement retrun_error on rust side and use reject(...) to communicate the error
            resolve(data);
          };
        }
      );

      this.zerdeEventloopEvents.callRust(name, params, callbackId);
      this.doWasmIo();
      return promise;
    };
    rpc.receive<
      { name: string; params: (string | PostMessageTypedArray | Uint8Array)[] },
      Promise<(string | BufferData)[]>
    >(WorkerEvent.CallRust, callRust);

    rpc.receive(WorkerEvent.IncrementArc, (arcPtr: number) => {
      this.exports.incrementArc(BigInt(arcPtr));
    });

    rpc.receive(WorkerEvent.DecrementArc, (arcPtr: number) => {
      this.exports.decrementArc(BigInt(arcPtr));
    });

    rpc.receive(
      WorkerEvent.DeallocVec,
      ({ bufferPtr, bufferLen, bufferCap }: BufferData) => {
        this.exports.deallocVec(
          BigInt(bufferPtr),
          BigInt(bufferLen),
          BigInt(bufferCap)
        );
      }
    );

    const bindUserWorkerPortOnMainThread = (port: MessagePort) => {
      const userWorkerRpc = new Rpc(port);
      userWorkerRpc.receive(
        UserWorkerEvent.Init,
        (): UserWorkerInitReturnValue => {
          return {
            wasmModule: this.module,
            memory: this.memory,
            taskWorkerSab,
            appPtr: this.appPtr,
            baseUri,
          };
        }
      );
      userWorkerRpc.receive(
        UserWorkerEvent.BindUserWorkerPortOnMainThread,
        ({ port }) => {
          bindUserWorkerPortOnMainThread(port);
        }
      );

      userWorkerRpc.receive<
        {
          name: string;
          params: (string | PostMessageTypedArray | Uint8Array)[];
        },
        Promise<(string | BufferData)[]>
      >(UserWorkerEvent.CallRust, callRust);
      userWorkerRpc.receive(
        UserWorkerEvent.SendEventFromAnyThread,
        (eventPtr: bigint) => {
          this.sendEventFromAnyThread(eventPtr);
        }
      );
    };
    rpc.receive(
      WorkerEvent.BindUserWorkerPortOnMainThread,
      (port: MessagePort) => {
        bindUserWorkerPortOnMainThread(port);
      }
    );

    // create initial zerdeEventloopEvents
    this.zerdeEventloopEvents = new ZerdeEventloopEvents(this);

    // fetch dependencies
    this.zerdeEventloopEvents.fetchDeps();

    this.doWasmIo();

    this.doWasmBlock = true;

    // ok now, we wait for our resources to load.
    Promise.all(this.resources).then(this.doDepResults.bind(this));
  }

  doDepResults(results: Resource[]): void {
    const deps: Dependency[] = [];
    // copy our reslts into wasm pointers
    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      // allocate pointer, do +8 because of the u64 length at the head of the buffer
      const vecLen = result.buffer.byteLength;
      const vecPtr = Number(this.zerdeEventloopEvents.allocWasmVec(vecLen));
      copyUint8ArrayToRustBuffer(
        new Uint8Array(result.buffer),
        this.zerdeEventloopEvents.getWasmApp().memory.buffer,
        vecPtr
      );
      deps.push({
        name: result.name,
        vecPtr,
        vecLen,
      });
    }
    // pass wasm the deps
    this.zerdeEventloopEvents.depsLoaded(deps);
    // initialize the application
    this.zerdeEventloopEvents.init({
      width: this.width,
      height: this.height,
      dpiFactor: this.dpiFactor,
      xrCanPresent: this.xrCanPresent,
      canFullscreen: this.canFullscreen,
      xrIsPresenting: false,
    });
    this.doWasmBlock = false;
    this.doWasmIo();

    rpc.send(WorkerEvent.RemoveLoadingIndicators, {});
  }

  doWasmIo(): void {
    if (this.doWasmBlock) {
      return;
    }

    const byteOffset = this.zerdeEventloopEvents.end();
    const zerdeParserPtr = Number(
      this.exports.processWasmEvents(this.appPtr, BigInt(byteOffset))
    );

    // get a clean zerdeEventloopEvents set up immediately
    this.zerdeEventloopEvents = new ZerdeEventloopEvents(this);
    this.zerdeParser = new ZerdeParser(this.memory, zerdeParserPtr);

    this.basef32 = new Float32Array(this.memory.buffer);
    this.baseu32 = new Uint32Array(this.memory.buffer);
    this.basef64 = new Float64Array(this.memory.buffer);
    this.baseu64 = new BigUint64Array(this.memory.buffer);

    // process all messages
    const sendFnTable = this.sendFnTable;

    // eslint-disable-next-line no-constant-condition
    while (1) {
      const msgType = this.zerdeParser.parseU32();
      if (sendFnTable[msgType](this)) {
        break;
      }
    }

    this.exports.deallocWasmMessage(BigInt(zerdeParserPtr));
  }

  // TODO(JP): Should use sychronous file loading for this.
  loadDeps(deps: string[]): void {
    for (let i = 0; i < deps.length; i++) {
      const filePath = deps[i];
      this.resources.push(this.fetchPath(filePath));
    }
  }

  setDocumentTitle(title: string): void {
    rpc.send(WorkerEvent.SetDocumentTitle, { title });
  }

  bindMouseAndTouch(): void {
    let lastMouseFinger;
    // TODO(JP): Some day bring back touch support..
    // let use_touch_scroll_overlay = window.ontouchstart === null;
    // if (use_touch_scroll_overlay) {
    //     var ts = this.touch_scroll_overlay = document.createElement('div')
    //     ts.className = "cx_webgl_scroll_overlay"
    //     var ts_inner = document.createElement('div')
    //     var style = document.createElement('style')
    //     style.innerHTML = "\n"
    //         + "div.cx_webgl_scroll_overlay {\n"
    //         + "z-index: 10000;\n"
    //         + "margin:0;\n"
    //         + "overflow:scroll;\n"
    //         + "top:0;\n"
    //         + "left:0;\n"
    //         + "width:100%;\n"
    //         + "height:100%;\n"
    //         + "position:fixed;\n"
    //         + "background-color:transparent\n"
    //         + "}\n"
    //         + "div.cx_webgl_scroll_overlay div{\n"
    //         + "margin:0;\n"
    //         + "width:400000px;\n"
    //         + "height:400000px;\n"
    //         + "background-color:transparent\n"
    //         + "}\n"

    //     document.body.appendChild(style)
    //     ts.appendChild(ts_inner);
    //     document.body.appendChild(ts);
    //     canvas = ts;

    //     ts.scrollTop = 200000;
    //     ts.scrollLeft = 200000;
    //     let last_scroll_top = ts.scrollTop;
    //     let last_scroll_left = ts.scrollLeft;
    //     let scroll_timeout = null;
    //     ts.addEventListener('scroll', e => {
    //         let new_scroll_top = ts.scrollTop;
    //         let new_scroll_left = ts.scrollLeft;
    //         let dx = new_scroll_left - last_scroll_left;
    //         let dy = new_scroll_top - last_scroll_top;
    //         last_scroll_top = new_scroll_top;
    //         last_scroll_left = new_scroll_left;
    //         self.clearTimeout(scroll_timeout);
    //         scroll_timeout = self.setTimeout(_ => {
    //             ts.scrollTop = 200000;
    //             ts.scrollLeft = 200000;
    //             last_scroll_top = ts.scrollTop;
    //             last_scroll_left = ts.scrollLeft;
    //         }, 200);

    //         let finger = last_mouse_finger;
    //         if (finger) {
    //             finger.scroll_x = dx;
    //             finger.scroll_y = dy;
    //             finger.is_wheel = true;
    //             this.zerdeEventloopEvents.finger_scroll(finger);
    //             this.do_wasm_io();
    //         }
    //     })
    // }

    const mouseFingers = [];
    function mouseToFinger(e: RpcEvent): Finger {
      const mf = mouseFingers[e.button] || (mouseFingers[e.button] = {});
      mf.x = e.pageX;
      mf.y = e.pageY;
      mf.digit = e.button;
      mf.time = performance.now() / 1000.0;
      mf.modifiers = packKeyModifier(e);
      mf.touch = false;
      return mf;
    }

    // var digit_map = {}
    // var digit_alloc = 0;

    // function touch_to_finger_alloc(e) {
    //     var f = []
    //     for (let i = 0; i < e.changedTouches.length; i ++) {
    //         var t = e.changedTouches[i]
    //         // find an unused digit
    //         var digit = undefined;
    //         for (digit in digit_map) {
    //             if (!digit_map[digit]) break
    //         }
    //         // we need to alloc a new one
    //         if (digit === undefined || digit_map[digit]) digit = digit_alloc ++;
    //         // store it
    //         digit_map[digit] = {identifier: t.identifier};
    //         // return allocated digit
    //         digit = parseInt(digit);

    //         f.push({
    //             x: t.pageX,
    //             y: t.pageY,
    //             digit: digit,
    //             time: e.timeStamp / 1000.0,
    //             modifiers: 0,
    //             touch: true,
    //         })
    //     }
    //     return f
    // }

    // function lookup_digit(identifier) {
    //     for (let digit in digit_map) {
    //         var digit_id = digit_map[digit]
    //         if (!digit_id) continue
    //         if (digit_id.identifier == identifier) {
    //             return digit
    //         }
    //     }
    // }

    // function touch_to_finger_lookup(e) {
    //     var f = []
    //     for (let i = 0; i < e.changedTouches.length; i ++) {
    //         var t = e.changedTouches[i]
    //         f.push({
    //             x: t.pageX,
    //             y: t.pageY,
    //             digit: lookup_digit(t.identifier),
    //             time: e.timeStamp / 1000.0,
    //             modifiers: {},
    //             touch: true,
    //         })
    //     }
    //     return f
    // }

    // function touch_to_finger_free(e) {
    //     var f = []
    //     for (let i = 0; i < e.changedTouches.length; i ++) {
    //         var t = e.changedTouches[i]
    //         var digit = lookup_digit(t.identifier)
    //         if (!digit) {
    //             console.log("Undefined state in free_digit");
    //             digit = 0
    //         }
    //         else {
    //             digit_map[digit] = undefined
    //         }

    //         f.push({
    //             x: t.pageX,
    //             y: t.pageY,
    //             time: e.timeStamp / 1000.0,
    //             digit: digit,
    //             modifiers: 0,
    //             touch: true,
    //         })
    //     }
    //     return f
    // }

    // var easy_xr_presenting_toggle = window.localStorage.getItem("xr_presenting") == "true"

    const mouseButtonsDown = [];
    rpc.receive(
      WorkerEvent.CanvasMouseDown,
      ({ event }: { event: RpcEvent }) => {
        mouseButtonsDown[event.button] = true;
        this.zerdeEventloopEvents.fingerDown(mouseToFinger(event));
        this.doWasmIo();
      }
    );

    rpc.receive(WorkerEvent.WindowMouseUp, ({ event }) => {
      mouseButtonsDown[event.button] = false;
      this.zerdeEventloopEvents.fingerUp(mouseToFinger(event));
      this.doWasmIo();
    });

    rpc.receive(WorkerEvent.WindowMouseMove, ({ event }) => {
      for (let i = 0; i < mouseButtonsDown.length; i++) {
        if (mouseButtonsDown[i]) {
          const mf = mouseToFinger(event);
          mf.digit = i;
          this.zerdeEventloopEvents.fingerMove(mf);
        }
      }
      lastMouseFinger = mouseToFinger(event);
      this.zerdeEventloopEvents.fingerHover(lastMouseFinger);
      this.doWasmIo();
      //console.log("Redraw cycle "+(end-begin)+" ms");
    });

    rpc.receive(WorkerEvent.WindowMouseOut, ({ event }) => {
      this.zerdeEventloopEvents.fingerOut(mouseToFinger(event)); //e.pageX, e.pageY, pa;
      this.doWasmIo();
    });
    // canvas.addEventListener('touchstart', e => {
    //     e.preventDefault()

    //     let fingers = touch_to_finger_alloc(e);
    //     for (let i = 0; i < fingers.length; i ++) {
    //         this.zerdeEventloopEvents.finger_down(fingers[i])
    //     }
    //     this.do_wasm_io();
    //     return false
    // })
    // canvas.addEventListener('touchmove', e => {
    //     //e.preventDefault();
    //     var fingers = touch_to_finger_lookup(e);
    //     for (let i = 0; i < fingers.length; i ++) {
    //         this.zerdeEventloopEvents.finger_move(fingers[i])
    //     }
    //     this.do_wasm_io();
    //     return false
    // }, {passive: false})

    // var end_cancel_leave = e => {
    //     //if (easy_xr_presenting_toggle) {
    //     //    easy_xr_presenting_toggle = false;
    //     //    this.xr_start_presenting();
    //     //};

    //     e.preventDefault();
    //     var fingers = touch_to_finger_free(e);
    //     for (let i = 0; i < fingers.length; i ++) {
    //         this.zerdeEventloopEvents.finger_up(fingers[i])
    //     }
    //     this.do_wasm_io();
    //     return false
    // }

    // canvas.addEventListener('touchend', end_cancel_leave);
    // canvas.addEventListener('touchcancel', end_cancel_leave);
    // canvas.addEventListener('touchleave', end_cancel_leave);

    let lastWheelTime: number;
    let lastWasWheel: boolean;
    rpc.receive(WorkerEvent.CanvasWheel, ({ event }: { event: RpcEvent }) => {
      const finger = mouseToFinger(event);
      const delta = event.timeStamp - lastWheelTime;
      lastWheelTime = event.timeStamp;
      // typical web bullshit. this reliably detects mousewheel or touchpad on mac in safari
      if (isFirefox) {
        lastWasWheel = event.deltaMode == 1;
      } else {
        // detect it
        if (
          Math.abs(Math.abs(event.deltaY / event.wheelDeltaY) - 1 / 3) <
            0.00001 ||
          (!lastWasWheel && delta < 250)
        ) {
          lastWasWheel = false;
        } else {
          lastWasWheel = true;
        }
      }
      //console.log(event.deltaY / event.wheelDeltaY);
      //last_delta = delta;
      let fac = 1;
      if (event.deltaMode === 1) {
        fac = 40;
      } else if (event.deltaMode === 2) {
        // TODO(Paras): deltaMode=2 means that a user is trying to scroll one page at a time.
        // For now, we hardcode the pixel amount. We can later determine this contextually.
        const offsetHeight = 800;
        fac = offsetHeight;
      }
      const fingerScroll = {
        ...finger,
        scrollX: event.deltaX * fac,
        scrollY: event.deltaY * fac,
        isWheel: lastWasWheel,
      };
      this.zerdeEventloopEvents.fingerScroll(fingerScroll);
      this.doWasmIo();
    });

    //window.addEventListener('webkitmouseforcewillbegin', this.onCheckMacForce.bind(this), false)
    //window.addEventListener('webkitmouseforcechanged', this.onCheckMacForce.bind(this), false)
  }

  bindKeyboard(): void {
    rpc.receive(WorkerEvent.TextInput, (data: TextareaEventTextInput) => {
      this.zerdeEventloopEvents.textInput(data);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.TextCopy, () => {
      this.zerdeEventloopEvents.textCopy();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.KeyDown, (data: TextareaEventKeyDown) => {
      this.zerdeEventloopEvents.keyDown(data);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.KeyUp, (data: TextareaEventKeyUp) => {
      this.zerdeEventloopEvents.keyUp(data);
      this.doWasmIo();
    });
  }

  setMouseCursor(id: number): void {
    rpc.send(WorkerEvent.SetMouseCursor, { style: cursorMap[id] || "default" });
  }

  startTimer(id: number, interval: number, repeats: number): void {
    for (let i = 0; i < this.timers.length; i++) {
      if (this.timers[i].id == id) {
        console.log("Timer ID collision!");
        return;
      }
    }
    const sysId =
      repeats !== 0
        ? self.setInterval(() => {
            this.zerdeEventloopEvents.timerFired(id);
            this.doWasmIo();
          }, interval * 1000.0)
        : self.setTimeout(() => {
            for (let i = 0; i < this.timers.length; i++) {
              const timer = this.timers[i];
              if (timer.id == id) {
                this.timers.splice(i, 1);
                break;
              }
            }
            this.zerdeEventloopEvents.timerFired(id);
            this.doWasmIo();
          }, interval * 1000.0);

    this.timers.push({ id, repeats, sysId });
  }

  stopTimer(id: number): void {
    for (let i = 0; i < this.timers.length; i++) {
      const timer = this.timers[i];
      if (timer.id == id) {
        if (timer.repeats) {
          self.clearInterval(timer.sysId);
        } else {
          self.clearTimeout(timer.sysId);
        }
        this.timers.splice(i, 1);
        return;
      }
    }
    //console.log("Timer ID not found!")
  }

  httpSend(
    verb: string,
    path: string,
    proto: string,
    domain: string,
    port: number,
    contentType: string,
    body: Uint8Array,
    signalId: number
  ): void {
    const req = new XMLHttpRequest();
    req.addEventListener("error", (_) => {
      // signal fail
      this.zerdeEventloopEvents.httpSendResponse(signalId, 2);
      this.doWasmIo();
    });
    req.addEventListener("load", (_) => {
      if (req.status !== 200) {
        // signal fail
        this.zerdeEventloopEvents.httpSendResponse(signalId, 2);
      } else {
        //signal success
        this.zerdeEventloopEvents.httpSendResponse(signalId, 1);
      }
      this.doWasmIo();
    });
    req.open(verb, proto + "://" + domain + ":" + port + path, true);
    console.log(verb, proto + "://" + domain + ":" + port + path, body);
    req.send(body.buffer);
  }

  websocketSend(url: string, data: Uint8Array): void {
    // TODO(Paras): Stop patching sendStack onto websockets
    // and maintain our own structure instead.
    const socket = this.websockets[url];
    if (!socket) {
      const socket = new WebSocket(url);
      this.websockets[url] = socket;
      // @ts-ignore
      socket.sendStack = [data];
      socket.addEventListener("close", () => {
        this.websockets[url] = null;
      });
      socket.addEventListener("error", (event) => {
        this.websockets[url] = null;
        this.zerdeEventloopEvents.websocketError(url, "" + event);
        this.doWasmIo();
      });
      socket.addEventListener("message", (event) => {
        event.data.arrayBuffer().then((data) => {
          this.zerdeEventloopEvents.websocketMessage(url, data);
          this.doWasmIo();
        });
      });
      socket.addEventListener("open", () => {
        // @ts-ignore
        const sendStack = socket.sendStack;
        // @ts-ignore
        socket.sendStack = null;
        for (data of sendStack) {
          socket.send(data);
        }
      });
    } else {
      if (socket.sendStack) {
        socket.sendStack.push(data);
      } else {
        socket.send(data);
      }
    }
  }

  enableGlobalFileDropTarget(): void {
    rpc.send(WorkerEvent.EnableGlobalFileDropTarget, {});
    rpc.receive(WorkerEvent.DragEnter, () => {
      this.zerdeEventloopEvents.dragenter();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragOver, ({ x, y }) => {
      this.zerdeEventloopEvents.dragover(x, y);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragLeave, () => {
      this.zerdeEventloopEvents.dragleave();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.Drop, ({ files }) => {
      const fileHandlesToSend = [];
      for (const file of files) {
        const fileHandle = {
          id: this.fileHandles.length,
          basename: file.name,
          file,
          lastReadStart: -1,
          lastReadEnd: -1,
        };
        fileHandlesToSend.push(fileHandle);
        this.fileHandles.push(fileHandle);
      }
      this.zerdeEventloopEvents.appOpenFiles(fileHandlesToSend);
      this.doWasmIo();
    });
  }

  initWebglContext(): void {
    rpc.receive(
      WorkerEvent.ScreenResize,
      ({ dpiFactor, width, height, isFullscreen }) => {
        this.dpiFactor = dpiFactor;
        this.width = width;
        this.height = height;

        this.canvas.width = width * dpiFactor;
        this.canvas.height = height * dpiFactor;
        this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);

        this.zerdeEventloopEvents.resize({
          width: this.width,
          height: this.height,
          dpiFactor: this.dpiFactor,
          xrIsPresenting: this.xrIsPresenting,
          xrCanPresent: this.xrCanPresent,
          isFullscreen: isFullscreen,
        });
        this.requestAnimationFrame();
      }
    );

    const options = {
      preferLowPowerToHighPerformance: true,
      // xrCompatible: true // TODO(JP): Bring back some day?
    };
    // @ts-ignore - TODO(Paras): Get proper support for OffscreenCanvas
    const gl = (this.gl =
      // @ts-ignore
      this.canvas.getContext("webgl", options) ||
      // @ts-ignore
      this.canvas.getContext("webgl-experimental", options) ||
      // @ts-ignore
      this.canvas.getContext("experimental-webgl", options));

    if (!gl) {
      rpc.send(WorkerEvent.ShowIncompatibleBrowserNotification, {});
      return;
    }
    this.OESStandardDerivatives = gl.getExtension("OES_standard_derivatives");
    this.OESVertexArrayObject = gl.getExtension("OES_vertex_array_object");
    this.OESElementIndexUint = gl.getExtension("OES_element_index_uint");
    this.ANGLEInstancedArrays = gl.getExtension("ANGLE_instanced_arrays");
  }

  requestAnimationFrame(): void {
    if (this.xrIsPresenting || this.reqAnimFrameId) {
      return;
    }
    this.reqAnimFrameId = self.requestAnimationFrame(() => {
      this.reqAnimFrameId = 0;
      if (this.xrIsPresenting) {
        return;
      }
      this.zerdeEventloopEvents.animationFrame();
      this.inAnimationFrame = true;
      this.doWasmIo();
      this.inAnimationFrame = false;
    });
  }

  runAsyncWebXRCheck(): void {
    this.xrCanPresent = false;
    this.xrIsPresenting = false;

    // ok this changes a bunch in how the renderflow works.
    // first thing we are going to do is get the vr displays.
    // @ts-ignore - Let's not worry about XR.
    const xrSystem = self.navigator.xr;
    if (xrSystem) {
      xrSystem.isSessionSupported("immersive-vr").then((supported) => {
        if (supported) {
          this.xrCanPresent = true;
        }
      });
    } else {
      console.log("No webVR support found");
    }
  }

  xrStartPresenting(): void {
    // TODO(JP): Some day bring back XR support?
    // if (this.xr_can_present) {
    //     navigator.xr.requestSession('immersive-vr', {requiredFeatures: ['local-floor']}).then(xr_session => {
    //         let xr_layer = new XRWebGLLayer(xr_session, this.gl, {
    //             antialias: false,
    //             depth: true,
    //             stencil: false,
    //             alpha: false,
    //             ignoreDepthValues: false,
    //             framebufferScaleFactor: 1.5
    //         });
    //         xr_session.updateRenderState({baseLayer: xr_layer});
    //         xr_session.requestReferenceSpace("local-floor").then(xr_reference_space => {
    //             window.localStorage.setItem("xr_presenting", "true");
    //             this.xr_reference_space = xr_reference_space;
    //             this.xr_session = xr_session;
    //             this.xr_is_presenting = true;
    //             let first_on_resize = true;
    //             // read shit off the gamepad
    //             xr_session.gamepad;
    //             // lets start the loop
    //             let inputs = [];
    //             let alternate = false;
    //             let last_time;
    //             let xr_on_request_animation_frame = (time, xr_frame) => {
    //                 if (first_on_resize) {
    //                     this.on_screen_resize();
    //                     first_on_resize = false;
    //                 }
    //                 if (!this.xr_is_presenting) {
    //                     return;
    //                 }
    //                 this.xr_session.requestAnimationFrame(xr_on_request_animation_frame);
    //                 this.xr_pose = xr_frame.getViewerPose(this.xr_reference_space);
    //                 if (!this.xr_pose) {
    //                     return;
    //                 }
    //                 this.zerdeEventloopEvents.xr_update_inputs(xr_frame, xr_session, time, this.xr_pose, this.xr_reference_space)
    //                 this.zerdeEventloopEvents.animation_frame(time / 1000.0);
    //                 this.in_animation_frame = true;
    //                 let start = performance.now();
    //                 this.do_wasm_io();
    //                 this.in_animation_frame = false;
    //                 this.xr_pose = null;
    //                 //let new_time = performance.now();
    //                 //if (new_time - last_time > 13.) {
    //                 //    console.log(new_time - last_time);
    //                 // }
    //                 //last_time = new_time;
    //             }
    //             this.xr_session.requestAnimationFrame(xr_on_request_animation_frame);
    //             this.xr_session.addEventListener("end", () => {
    //                 window.localStorage.setItem("xr_presenting", "false");
    //                 this.xr_is_presenting = false;
    //                 this.on_screen_resize();
    //                 this.zerdeEventloopEvents.paint_dirty();
    //                 this.request_animation_frame();
    //             })
    //         })
    //     })
    // }
  }

  xrStopPresenting(): void {
    // ignore for now
  }

  beginMainCanvas(
    r: number,
    g: number,
    b: number,
    a: number,
    depth: number
  ): void {
    const gl = this.gl;
    this.isMainCanvas = true;
    if (this.xrIsPresenting) {
      // let xr_webgllayer = this.xr_session.renderState.baseLayer;
      // this.gl.bindFramebuffer(gl.FRAMEBUFFER, xr_webgllayer.framebuffer);
      // gl.viewport(0, 0, xr_webgllayer.framebufferWidth, xr_webgllayer.framebufferHeight);
      // // quest 1 is 3648
      // // quest 2 is 4096
      // let left_view = this.xr_pose.views[0];
      // let right_view = this.xr_pose.views[1];
      // this.xr_left_viewport = xr_webgllayer.getViewport(left_view);
      // this.xr_right_viewport = xr_webgllayer.getViewport(right_view);
      // this.xr_left_projection_matrix = left_view.projectionMatrix;
      // this.xr_left_transform_matrix = left_view.transform.inverse.matrix;
      // this.xr_left_invtransform_matrix = left_view.transform.matrix;
      // this.xr_right_projection_matrix = right_view.projectionMatrix;
      // this.xr_right_transform_matrix = right_view.transform.inverse.matrix;
      // this.xr_right_camera_pos = right_view.transform.inverse.position;
      // this.xr_right_invtransform_matrix = right_view.transform.matrix;
    } else {
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    }

    gl.clearColor(r, g, b, a);
    gl.clearDepth(depth);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  }

  beginRenderTargets(passId: number, width: number, height: number): void {
    const gl = this.gl;
    this.targetWidth = width;
    this.targetHeight = height;
    this.colorTargets = 0;
    this.clearFlags = 0;
    this.isMainCanvas = false;
    const glFramebuffer =
      this.framebuffers[passId] ||
      (this.framebuffers[passId] = gl.createFramebuffer());
    gl.bindFramebuffer(gl.FRAMEBUFFER, glFramebuffer);
  }

  addColorTarget(
    textureId: number,
    initOnly: number,
    r: number,
    g: number,
    b: number,
    a: number
  ): void {
    // if use_default
    this.clearR = r;
    this.clearG = g;
    this.clearB = b;
    this.clearA = a;
    const gl = this.gl;

    const glTex =
      this.textures[textureId] ||
      (this.textures[textureId] = gl.createTexture() as Texture);

    // resize or create texture
    if (
      glTex.mpWidth != this.targetWidth ||
      glTex.mpHeight != this.targetHeight
    ) {
      gl.bindTexture(gl.TEXTURE_2D, glTex);
      this.clearFlags |= gl.COLOR_BUFFER_BIT;

      glTex.mpWidth = this.targetWidth;
      glTex.mpHeight = this.targetHeight;
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        glTex.mpWidth,
        glTex.mpHeight,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        null
      );
    } else if (!initOnly) {
      this.clearFlags |= gl.COLOR_BUFFER_BIT;
    }

    gl.framebufferTexture2D(
      gl.FRAMEBUFFER,
      gl.COLOR_ATTACHMENT0,
      gl.TEXTURE_2D,
      glTex,
      0
    );
    // TODO(Shobhit) - color_targets never gets used, maybe we should remove it in future
    this.colorTargets += 1;
  }

  setDepthTarget(textureId: number, initOnly: number, depth: number): void {
    const gl = this.gl;
    this.clearDepth = depth;

    const glRenderBuffer =
      this.textures[textureId] ||
      (this.textures[textureId] = gl.createRenderbuffer() as Texture);

    if (
      glRenderBuffer.mpWidth != this.targetWidth ||
      glRenderBuffer.mpHeight != this.targetHeight
    ) {
      // Borrowed concept from https://webglfundamentals.org/webgl/lessons/webgl-render-to-texture.html
      gl.bindRenderbuffer(gl.RENDERBUFFER, glRenderBuffer);
      this.clearFlags |= gl.DEPTH_BUFFER_BIT;
      glRenderBuffer.mpWidth = this.targetWidth;
      glRenderBuffer.mpHeight = this.targetHeight;
      gl.renderbufferStorage(
        gl.RENDERBUFFER,
        gl.DEPTH_COMPONENT16,
        this.targetWidth,
        this.targetHeight
      );
    } else if (!initOnly) {
      this.clearFlags |= gl.DEPTH_BUFFER_BIT;
    }
    gl.framebufferRenderbuffer(
      gl.FRAMEBUFFER,
      gl.DEPTH_ATTACHMENT,
      gl.RENDERBUFFER,
      glRenderBuffer
    );
  }

  endRenderTargets(): void {
    const gl = this.gl;

    // process the actual 'clear'
    gl.viewport(0, 0, this.targetWidth, this.targetHeight);

    // check if we need to clear color, and depth
    // clear it
    if (this.clearFlags) {
      gl.clearColor(this.clearR, this.clearG, this.clearB, this.clearA);
      gl.clearDepth(this.clearDepth);
      gl.clear(this.clearFlags);
    }
  }

  setDefaultDepthAndBlendMode(): void {
    const gl = this.gl;
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.blendEquationSeparate(gl.FUNC_ADD, gl.FUNC_ADD);
    gl.blendFuncSeparate(
      gl.ONE,
      gl.ONE_MINUS_SRC_ALPHA,
      gl.ONE,
      gl.ONE_MINUS_SRC_ALPHA
    );
    gl.enable(gl.BLEND);
  }

  // new shader helpers
  getAttribLocations(
    program: WebGLProgram,
    base: string,
    slots: number
  ): {
    loc: number;
    offset: number;
    size: number;
    stride: number;
  }[] {
    const gl = this.gl;
    const attribLocs = [];
    let attribs = slots >> 2;
    if ((slots & 3) != 0) attribs++;
    for (let i = 0; i < attribs; i++) {
      let size = slots - i * 4;
      if (size > 4) size = 4;
      attribLocs.push({
        loc: gl.getAttribLocation(program, base + i),
        offset: i * 16,
        size: size,
        stride: slots * 4,
      });
    }
    return attribLocs;
  }

  getUniformLocations(
    program: WebGLProgram,
    uniforms: Uniform[]
  ): {
    name: string;
    offset: number;
    ty: string;
    loc: WebGLUniformLocation;
    fn: WasmApp["uniformFnTable"][number];
  }[] {
    const gl = this.gl;
    const uniformLocs: {
      name: string;
      offset: number;
      ty: string;
      loc: WebGLUniformLocation;
      fn: WasmApp["uniformFnTable"][number];
    }[] = [];
    let offset = 0;
    for (let i = 0; i < uniforms.length; i++) {
      const uniform = uniforms[i];
      // lets align the uniform
      const slots = uniformSizeTable[uniform.ty];

      if ((offset & 3) != 0 && (offset & 3) + slots > 4) {
        // goes over the boundary
        offset += 4 - (offset & 3); // make jump to new slot
      }
      uniformLocs.push({
        name: uniform.name,
        offset: offset << 2,
        ty: uniform.ty,
        loc: gl.getUniformLocation(program, uniform.name),
        fn: this.uniformFnTable[uniform.ty],
      });
      offset += slots;
    }
    return uniformLocs;
  }

  compileWebGLShader(ash: ShaderAttributes): void {
    const gl = this.gl;
    const vsh = gl.createShader(gl.VERTEX_SHADER);

    gl.shaderSource(vsh, ash.vertex);
    gl.compileShader(vsh);
    if (!gl.getShaderParameter(vsh, gl.COMPILE_STATUS)) {
      console.log(gl.getShaderInfoLog(vsh), addLineNumbersToString(ash.vertex));
    }

    // compile pixelshader
    const fsh = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fsh, ash.fragment);
    gl.compileShader(fsh);
    if (!gl.getShaderParameter(fsh, gl.COMPILE_STATUS)) {
      console.log(
        gl.getShaderInfoLog(fsh),
        addLineNumbersToString(ash.fragment)
      );
    }

    const program = gl.createProgram();
    gl.attachShader(program, vsh);
    gl.attachShader(program, fsh);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.log(
        gl.getProgramInfoLog(program),
        addLineNumbersToString(ash.vertex),
        addLineNumbersToString(ash.fragment)
      );
    }
    // fetch all attribs and uniforms
    this.shaders[ash.shaderId] = {
      geomAttribs: this.getAttribLocations(
        program,
        "mpsc_packed_geometry_",
        ash.geometrySlots
      ),
      instAttribs: this.getAttribLocations(
        program,
        "mpsc_packed_instance_",
        ash.instanceSlots
      ),
      passUniforms: this.getUniformLocations(program, ash.passUniforms),
      viewUniforms: this.getUniformLocations(program, ash.viewUniforms),
      drawUniforms: this.getUniformLocations(program, ash.drawUniforms),
      userUniforms: this.getUniformLocations(program, ash.userUniforms),
      textureSlots: this.getUniformLocations(program, ash.textureSlots),
      instanceSlots: ash.instanceSlots,
      program: program,
      ash: ash,
    };
  }

  allocArrayBuffer(arrayBufferId: number, array: Float32Array): void {
    const gl = this.gl;
    let buf = this.arrayBuffers[arrayBufferId];
    if (buf === undefined) {
      buf = this.arrayBuffers[arrayBufferId] = {
        glBuf: gl.createBuffer(),
        length: array.length,
      };
    } else {
      buf.length = array.length;
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, buf.glBuf);
    gl.bufferData(gl.ARRAY_BUFFER, array, gl.STATIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  allocIndexBuffer(indexBufferId: number, array: Uint32Array): void {
    const gl = this.gl;

    let buf = this.indexBuffers[indexBufferId];
    if (buf === undefined) {
      buf = this.indexBuffers[indexBufferId] = {
        glBuf: gl.createBuffer(),
        length: array.length,
      };
    } else {
      buf.length = array.length;
    }
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buf.glBuf);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, array, gl.STATIC_DRAW);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
  }

  allocTexture(
    textureId: number,
    width: number,
    height: number,
    dataPtr: number
  ): void {
    const gl = this.gl;
    const glTex = this.textures[textureId] || gl.createTexture();

    gl.bindTexture(gl.TEXTURE_2D, glTex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    const data = new Uint8Array(
      this.memory.buffer,
      dataPtr,
      width * height * 4
    );
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      width,
      height,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      data
    );
    this.textures[textureId] = glTex as Texture;
  }

  allocVao(
    vaoId: number,
    shaderId: number,
    geomIbId: number,
    geomVbId: number,
    instVbId: number
  ): void {
    const gl = this.gl;
    const oldVao = this.vaos[vaoId];
    if (oldVao) {
      this.OESVertexArrayObject.deleteVertexArrayOES(oldVao.glVao);
    }
    const glVao = this.OESVertexArrayObject.createVertexArrayOES();
    const vao = (this.vaos[vaoId] = { glVao, geomIbId, geomVbId, instVbId });

    this.OESVertexArrayObject.bindVertexArrayOES(vao.glVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.arrayBuffers[geomVbId].glBuf);

    const shader = this.shaders[shaderId];

    for (let i = 0; i < shader.geomAttribs.length; i++) {
      const attr = shader.geomAttribs[i];
      if (attr.loc < 0) {
        continue;
      }
      gl.vertexAttribPointer(
        attr.loc,
        attr.size,
        gl.FLOAT,
        false,
        attr.stride,
        attr.offset
      );
      gl.enableVertexAttribArray(attr.loc);
      this.ANGLEInstancedArrays.vertexAttribDivisorANGLE(attr.loc, 0);
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, this.arrayBuffers[instVbId].glBuf);
    for (let i = 0; i < shader.instAttribs.length; i++) {
      const attr = shader.instAttribs[i];
      if (attr.loc < 0) {
        continue;
      }
      gl.vertexAttribPointer(
        attr.loc,
        attr.size,
        gl.FLOAT,
        false,
        attr.stride,
        attr.offset
      );
      gl.enableVertexAttribArray(attr.loc);
      this.ANGLEInstancedArrays.vertexAttribDivisorANGLE(attr.loc, 1);
    }

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffers[geomIbId].glBuf);
    this.OESVertexArrayObject.bindVertexArrayOES(null);
  }

  drawCall(
    shaderId: number,
    vaoId: number,
    passUniformsPtr: number,
    viewUniformsPtr: number,
    drawUniformsPtr: number,
    userUniformsPtr: number,
    texturesPtr: number
  ): void {
    const gl = this.gl;

    const shader = this.shaders[shaderId];
    gl.useProgram(shader.program);

    const vao = this.vaos[vaoId];

    this.OESVertexArrayObject.bindVertexArrayOES(vao.glVao);

    const indexBuffer = this.indexBuffers[vao.geomIbId];
    const instanceBuffer = this.arrayBuffers[vao.instVbId];
    // set up uniforms TODO do this a bit more incremental based on uniform layer
    // also possibly use webGL2 uniform buffers. For now this will suffice for webGL 1 compat
    const passUniforms = shader.passUniforms;
    // if vr_presenting

    const viewUniforms = shader.viewUniforms;
    for (let i = 0; i < viewUniforms.length; i++) {
      const uni = viewUniforms[i];
      uni.fn(this, uni.loc, uni.offset + viewUniformsPtr);
    }
    const drawUniforms = shader.drawUniforms;
    for (let i = 0; i < drawUniforms.length; i++) {
      const uni = drawUniforms[i];
      uni.fn(this, uni.loc, uni.offset + drawUniformsPtr);
    }
    const userUniforms = shader.userUniforms;
    for (let i = 0; i < userUniforms.length; i++) {
      const uni = userUniforms[i];
      uni.fn(this, uni.loc, uni.offset + userUniformsPtr);
    }
    const textureSlots = shader.textureSlots;
    for (let i = 0; i < textureSlots.length; i++) {
      const texSlot = textureSlots[i];
      const texId = this.baseu32[(texturesPtr >> 2) + i];
      const texObj = this.textures[texId];
      gl.activeTexture(gl.TEXTURE0 + i);
      gl.bindTexture(gl.TEXTURE_2D, texObj);
      gl.uniform1i(texSlot.loc, i);
    }
    const indices = indexBuffer.length;
    const instances = instanceBuffer.length / shader.instanceSlots;
    // lets do a drawcall!

    if (this.isMainCanvas && this.xrIsPresenting) {
      // for (let i = 3; i < pass_uniforms.length; i ++) {
      //     let uni = pass_uniforms[i];
      //     uni.fn(this, uni.loc, uni.offset + pass_uniforms_ptr);
      // }
      // // the first 2 matrices are project and view
      // let left_viewport = this.xr_left_viewport;
      // gl.viewport(left_viewport.x, left_viewport.y, left_viewport.width, left_viewport.height);
      // gl.uniformMatrix4fv(pass_uniforms[0].loc, false, this.xr_left_projection_matrix);
      // gl.uniformMatrix4fv(pass_uniforms[1].loc, false, this.xr_left_transform_matrix);
      // gl.uniformMatrix4fv(pass_uniforms[2].loc, false, this.xr_left_invtransform_matrix);
      // this.ANGLE_instanced_arrays.drawElementsInstancedANGLE(gl.TRIANGLES, indices, gl.UNSIGNED_INT, 0, instances);
      // let right_viewport = this.xr_right_viewport;
      // gl.viewport(right_viewport.x, right_viewport.y, right_viewport.width, right_viewport.height);
      // gl.uniformMatrix4fv(pass_uniforms[0].loc, false, this.xr_right_projection_matrix);
      // gl.uniformMatrix4fv(pass_uniforms[1].loc, false, this.xr_right_transform_matrix);
      // gl.uniformMatrix4fv(pass_uniforms[2].loc, false, this.xr_right_invtransform_matrix);
      // this.ANGLE_instanced_arrays.drawElementsInstancedANGLE(gl.TRIANGLES, indices, gl.UNSIGNED_INT, 0, instances);
    } else {
      for (let i = 0; i < passUniforms.length; i++) {
        const uni = passUniforms[i];
        uni.fn(this, uni.loc, uni.offset + passUniformsPtr);
      }
      this.ANGLEInstancedArrays.drawElementsInstancedANGLE(
        gl.TRIANGLES,
        indices,
        gl.UNSIGNED_INT,
        0,
        instances
      );
    }
    this.OESVertexArrayObject.bindVertexArrayOES(null);
  }

  // TODO(JP): Should use sychronous file loading for this.
  fetchPath(filePath: string): Promise<Resource> {
    return new Promise((resolve, reject) => {
      const req = new XMLHttpRequest();
      req.addEventListener("error", function () {
        reject(filePath);
      });
      req.responseType = "arraybuffer";
      req.addEventListener("load", function () {
        if (req.status !== 200) {
          reject(req.status);
        }
        resolve({
          name: filePath,
          buffer: req.response,
        });
      });
      req.open("GET", new URL(filePath, this.baseUri).href);
      req.send();
    });
  }

  sendEventFromAnyThread(eventPtr: bigint): void {
    // Prevent an infinite loop when calling this from an event handler.
    setTimeout(() => {
      this.zerdeEventloopEvents.sendEventFromAnyThread(eventPtr);
      this.doWasmIo();
    });
  }
}

// array of function id's wasm can call on us, self is pointer to WasmApp
WasmApp.prototype.sendFnTable = [
  function end0(_self) {
    return true;
  },
  function log1(self) {
    console.log(self.zerdeParser.parseString());
  },
  function compileWebGLShader2(self) {
    function parseShvarvec(): Uniform[] {
      const len = self.zerdeParser.parseU32();
      const vars: Uniform[] = [];
      for (let i = 0; i < len; i++) {
        vars.push({
          ty: self.zerdeParser.parseString(),
          name: self.zerdeParser.parseString(),
        });
      }
      return vars;
    }

    const ash = {
      shaderId: self.zerdeParser.parseU32(),
      fragment: self.zerdeParser.parseString(),
      vertex: self.zerdeParser.parseString(),
      geometrySlots: self.zerdeParser.parseU32(),
      instanceSlots: self.zerdeParser.parseU32(),
      passUniforms: parseShvarvec(),
      viewUniforms: parseShvarvec(),
      drawUniforms: parseShvarvec(),
      userUniforms: parseShvarvec(),
      textureSlots: parseShvarvec(),
    };
    self.compileWebGLShader(ash);
  },
  function allocArrayBuffer3(self) {
    const arrayBufferId = self.zerdeParser.parseU32();
    const len = self.zerdeParser.parseU32();
    const pointer = self.zerdeParser.parseU32();
    const array = new Float32Array(self.memory.buffer, pointer, len);
    self.allocArrayBuffer(arrayBufferId, array);
  },
  function allocIndexBuffer4(self) {
    const indexBufferId = self.zerdeParser.parseU32();
    const len = self.zerdeParser.parseU32();
    const pointer = self.zerdeParser.parseU32();
    const array = new Uint32Array(self.memory.buffer, pointer, len);
    self.allocIndexBuffer(indexBufferId, array);
  },
  function allocVao5(self) {
    const vaoId = self.zerdeParser.parseU32();
    const shaderId = self.zerdeParser.parseU32();
    const geomIbId = self.zerdeParser.parseU32();
    const geomVbId = self.zerdeParser.parseU32();
    const instVbId = self.zerdeParser.parseU32();
    self.allocVao(vaoId, shaderId, geomIbId, geomVbId, instVbId);
  },
  function drawCall6(self) {
    const shaderId = self.zerdeParser.parseU32();
    const vaoId = self.zerdeParser.parseU32();
    const uniformsPassPtr = self.zerdeParser.parseU32();
    const uniformsViewPtr = self.zerdeParser.parseU32();
    const uniformsDrawPtr = self.zerdeParser.parseU32();
    const uniformsUserPtr = self.zerdeParser.parseU32();
    const textures = self.zerdeParser.parseU32();
    self.drawCall(
      shaderId,
      vaoId,
      uniformsPassPtr,
      uniformsViewPtr,
      uniformsDrawPtr,
      uniformsUserPtr,
      textures
    );
  },
  function unused7(_self) {
    // unused
  },
  function loadDeps8(self) {
    const deps: string[] = [];
    const numDeps = self.zerdeParser.parseU32();
    for (let i = 0; i < numDeps; i++) {
      deps.push(self.zerdeParser.parseString());
    }
    self.loadDeps(deps);
  },
  function allocTexture9(self) {
    const textureId = self.zerdeParser.parseU32();
    const width = self.zerdeParser.parseU32();
    const height = self.zerdeParser.parseU32();
    const dataPtr = self.zerdeParser.parseU32();
    self.allocTexture(textureId, width, height, dataPtr);
  },
  function requestAnimationFrame10(self) {
    self.requestAnimationFrame();
  },
  function setDocumentTitle11(self) {
    self.setDocumentTitle(self.zerdeParser.parseString());
  },
  function setMouseCursor12(self) {
    self.setMouseCursor(self.zerdeParser.parseU32());
  },
  function unused13(_self) {
    // unused
  },
  function showTextIme14(self) {
    const x = self.zerdeParser.parseF32();
    const y = self.zerdeParser.parseF32();
    rpc.send(WorkerEvent.ShowTextIME, { x, y });
  },
  function hideTextIme15(_self) {
    // TODO(JP): doesn't seem to do anything, is that intentional?
  },
  function textCopyResponse16(self) {
    const textCopyResponse = self.zerdeParser.parseString();
    rpc.send(WorkerEvent.TextCopyResponse, { textCopyResponse });
  },
  function startTimer17(self) {
    const repeats = self.zerdeParser.parseU32();
    const id = self.zerdeParser.parseF64();
    const interval = self.zerdeParser.parseF64();
    self.startTimer(id, interval, repeats);
  },
  function stopTimer18(self) {
    const id = self.zerdeParser.parseF64();
    self.stopTimer(id);
  },
  function xrStartPresenting19(self) {
    self.xrStartPresenting();
  },
  function xrStopPresenting20(self) {
    self.xrStopPresenting();
  },
  function beginRenderTargets21(self) {
    const passId = self.zerdeParser.parseU32();
    const width = self.zerdeParser.parseU32();
    const height = self.zerdeParser.parseU32();
    self.beginRenderTargets(passId, width, height);
  },
  function addColorTarget22(self) {
    const textureId = self.zerdeParser.parseU32();
    const initOnly = self.zerdeParser.parseU32();
    const r = self.zerdeParser.parseF32();
    const g = self.zerdeParser.parseF32();
    const b = self.zerdeParser.parseF32();
    const a = self.zerdeParser.parseF32();
    self.addColorTarget(textureId, initOnly, r, g, b, a);
  },
  function setDepthTarget23(self) {
    const textureId = self.zerdeParser.parseU32();
    const initOnly = self.zerdeParser.parseU32();
    const depth = self.zerdeParser.parseF32();
    self.setDepthTarget(textureId, initOnly, depth);
  },
  function endRenderTargets24(self) {
    self.endRenderTargets();
  },
  function setDefaultDepthAndBlendMode25(self) {
    self.setDefaultDepthAndBlendMode();
  },
  function beginMainCanvas26(self) {
    const r = self.zerdeParser.parseF32();
    const g = self.zerdeParser.parseF32();
    const b = self.zerdeParser.parseF32();
    const a = self.zerdeParser.parseF32();
    const depth = self.zerdeParser.parseF32();
    self.beginMainCanvas(r, g, b, a, depth);
  },
  function httpSend27(self) {
    const port = self.zerdeParser.parseU32();
    const signalId = self.zerdeParser.parseU32();
    const verb = self.zerdeParser.parseString();
    const path = self.zerdeParser.parseString();
    const proto = self.zerdeParser.parseString();
    const domain = self.zerdeParser.parseString();
    const contentType = self.zerdeParser.parseString();
    const body = self.zerdeParser.parseU8Slice();
    // do XHR.
    self.httpSend(verb, path, proto, domain, port, contentType, body, signalId);
  },
  function fullscreen28(_self) {
    rpc.send(WorkerEvent.Fullscreen);
  },
  function normalscreen29(_self) {
    rpc.send(WorkerEvent.Normalscreen);
  },
  function websocketSend30(self) {
    const url = self.zerdeParser.parseString();
    const data = self.zerdeParser.parseU8Slice();
    self.websocketSend(url, data);
  },
  function enableGlobalFileDropTarget31(self) {
    self.enableGlobalFileDropTarget();
  },
  function callJs32(self) {
    const fnName = self.zerdeParser.parseString();
    const params = self.zerdeParser.parseWrfParams();
    if (fnName === "_wrflibReturnParams") {
      const callbackId = JSON.parse(params[0] as string);
      self.callRustPendingCallbacks[callbackId](params.slice(1));
      delete self.callRustPendingCallbacks[callbackId];
    } else {
      const data: CallJSData = { fnName: fnName, params };
      rpc.send(WorkerEvent.CallJs, data);
    }
  },
];

WasmApp.prototype.uniformFnTable = {
  float: function setFloat(self, loc, off) {
    const slot = off >> 2;
    self.gl.uniform1f(loc, self.basef32[slot]);
  },
  vec2: function setVec2(self, loc, off) {
    const slot = off >> 2;
    const basef32 = self.basef32;
    self.gl.uniform2f(loc, basef32[slot], basef32[slot + 1]);
  },
  vec3: function setVec3(self, loc, off) {
    const slot = off >> 2;
    const basef32 = self.basef32;
    self.gl.uniform3f(loc, basef32[slot], basef32[slot + 1], basef32[slot + 2]);
  },
  vec4: function setVec4(self, loc, off) {
    const slot = off >> 2;
    const basef32 = self.basef32;
    self.gl.uniform4f(
      loc,
      basef32[slot],
      basef32[slot + 1],
      basef32[slot + 2],
      basef32[slot + 3]
    );
  },
  mat2: function setMat2(self, loc, off) {
    self.gl.uniformMatrix2fv(
      loc,
      false,
      new Float32Array(self.memory.buffer, off, 4)
    );
  },
  mat3: function setMat3(self, loc, off) {
    self.gl.uniformMatrix3fv(
      loc,
      false,
      new Float32Array(self.memory.buffer, off, 9)
    );
  },
  mat4: function setMat4(self, loc, off) {
    const mat4 = new Float32Array(self.memory.buffer, off, 16);
    self.gl.uniformMatrix4fv(loc, false, mat4);
  },
};

const uniformSizeTable = {
  float: 1,
  vec2: 2,
  vec3: 3,
  vec4: 4,
  mat2: 4,
  mat3: 9,
  mat4: 16,
};

function addLineNumbersToString(code) {
  const lines = code.split("\n");
  let out = "";
  for (let i = 0; i < lines.length; i++) {
    out += i + 1 + ": " + lines[i] + "\n";
  }
  return out;
}

rpc.receive(
  WorkerEvent.Init,
  ({ offscreenCanvas, wasmFilename, canFullscreen, baseUri, memory }) => {
    const wasmPath = new URL(wasmFilename, baseUri).href;

    let wasmapp;
    return new Promise<void>((resolve, reject) => {
      // TODO(JP): These file handles are only sent to a worker when it starts running;
      // it currently can't receive any file handles added after that.
      const fileHandles = [];

      const taskWorkerSab = initTaskWorkerSab();
      const taskWorker = new Worker(
        new URL("./task_worker.ts", import.meta.url)
      );
      const taskWorkerRpc = new Rpc(taskWorker);
      taskWorkerRpc.send(TaskWorkerEvent.Init, { taskWorkerSab, memory });

      const threadSpawn = (ctxPtr: BigInt) => {
        const worker = new Worker(
          new URL("./async_worker.ts", import.meta.url)
        );
        const workerRpc = new Rpc(worker);

        workerRpc.receive(
          AsyncWorkerEvent.SendEventFromAnyThread,
          ({ eventPtr }: { eventPtr: BigInt }) => {
            wasmapp.sendEventFromAnyThread(eventPtr);
          }
        );

        workerRpc.receive(
          AsyncWorkerEvent.ThreadSpawn,
          (data: { ctxPtr: BigInt }) => {
            threadSpawn(data.ctxPtr);
          }
        );

        workerRpc
          .send(AsyncWorkerEvent.Run, {
            wasmModule: wasmapp.module,
            memory,
            taskWorkerSab,
            ctxPtr,
            fileHandles,
            baseUri,
          })
          .catch((e) => {
            console.error("async worker failed", e);
          })
          .finally(() => {
            worker.terminate();
          });
      };

      const getExports = () => {
        return wasmapp.exports;
      };

      const env = getWasmEnv({
        getExports,
        memory,
        taskWorkerSab,
        fileHandles,
        sendEventFromAnyThread: (eventPtr: BigInt) => {
          wasmapp.sendEventFromAnyThread(eventPtr);
        },
        threadSpawn,
        baseUri,
      });

      WebAssembly.instantiateStreaming(fetch(wasmPath), { env }).then(
        (webasm) => {
          wasmapp = new WasmApp({
            offscreenCanvas,
            webasm,
            memory,
            canFullscreen,
            baseUri,
            fileHandles,
            taskWorkerSab,
          });
          resolve();
        },
        reject
      );
    });
  }
);
