// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

import { cursorMap } from "./cursor_map";
import {
  Rpc,
  getWasmEnv,
  makeThreadLocalStorageAndStackDataOnExistingThread,
  initThreadLocalStorageMainWorker,
} from "./common";
import {
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "./make_textarea";
import {
  FileHandle,
  WasmExports,
  PostMessageTypedArray,
  SizingData,
  WrfArray,
  MutableBufferData,
  RustWrfParam,
} from "./types";
import { ZerdeParser } from "./zerde";
import { ZerdeEventloopEvents } from "./zerde_eventloop_events";
import { packKeyModifier } from "./zerde_keyboard_handlers";
import { WebGLRenderer } from "./webgl_renderer";
import { RpcMouseEvent, RpcTouchEvent, RpcWheelEvent } from "./make_rpc_event";
import {
  Worker,
  WasmWorkerRpc,
  WebWorkerRpc,
  WorkerEvent,
  MainWorkerChannelEvent,
} from "./rpc_types";

const rpc = new Rpc<Worker<WasmWorkerRpc>>(self);

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
  button: number;
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

// TODO(Paras): Stop patching sendStack onto websockets
// and maintain our own structure instead.
type WebSocketWithSendStack = WebSocket & {
  sendStack?: Uint8Array[] | null;
};

export class WasmApp {
  memory: WebAssembly.Memory;
  exports: WasmExports;
  module: WebAssembly.Module;
  private sizingData: SizingData;
  private baseUri: string;
  private timers: Timer[];
  private resources: Promise<Resource>[];
  private hasRequestedAnimationFrame: boolean;
  private websockets: Record<string, WebSocketWithSendStack | null>;
  private fileHandles: FileHandle[];
  private zerdeEventloopEvents: ZerdeEventloopEvents;
  private appPtr: BigInt;
  private doWasmBlock: boolean;
  private xrCanPresent = false;
  private xrIsPresenting = false;
  private zerdeParser!: ZerdeParser;
  private callRustNewCallbackId: number;
  private callRustPendingCallbacks: Record<
    number,
    (arg0: RustWrfParam[]) => void
  >;
  // WebGLRenderer if we're using an OffscreenCanvas. If not, this is undefined.
  private webglRenderer: WebGLRenderer | undefined;
  // Promise which is set when we have an active RunWebGL call in the main browser thread.
  private runWebGLPromise: Promise<void> | undefined;

  constructor({
    offscreenCanvas,
    wasmModule,
    wasmExports,
    memory,
    sizingData,
    baseUri,
    fileHandles,
    taskWorkerSab,
  }: {
    offscreenCanvas: OffscreenCanvas | undefined;
    wasmModule: WebAssembly.Module;
    wasmExports: WasmExports;
    memory: WebAssembly.Memory;
    sizingData: SizingData;
    baseUri: string;
    fileHandles: FileHandle[];
    taskWorkerSab: SharedArrayBuffer;
  }) {
    this.module = wasmModule;
    this.exports = wasmExports;
    this.memory = memory;
    this.baseUri = baseUri;
    this.sizingData = sizingData;

    this.timers = [];
    this.resources = [];
    this.hasRequestedAnimationFrame = false;
    this.websockets = {};
    this.fileHandles = fileHandles;

    this.callRustNewCallbackId = 0;
    this.callRustPendingCallbacks = {};

    if (offscreenCanvas) {
      this.webglRenderer = new WebGLRenderer(
        offscreenCanvas,
        this.memory,
        this.sizingData,
        () => {
          rpc.send(WorkerEvent.ShowIncompatibleBrowserNotification);
        }
      );
    }

    rpc.receive(WorkerEvent.ScreenResize, (sizingData: SizingData) => {
      this.sizingData = sizingData;
      if (this.webglRenderer) {
        this.webglRenderer.resize(this.sizingData);
      }

      this.zerdeEventloopEvents.resize({
        width: this.sizingData.width,
        height: this.sizingData.height,
        dpiFactor: this.sizingData.dpiFactor,
        xrIsPresenting: this.xrIsPresenting,
        xrCanPresent: this.xrCanPresent,
        isFullscreen: this.sizingData.isFullscreen,
      });
      this.requestAnimationFrame();
    });

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
      params: (string | PostMessageTypedArray | WrfArray)[];
    }): Promise<RustWrfParam[]> => {
      const callbackId = this.callRustNewCallbackId++;
      const promise = new Promise<RustWrfParam[]>((resolve, _reject) => {
        this.callRustPendingCallbacks[callbackId] = (data: RustWrfParam[]) => {
          // TODO(Dmitry): implement retrun_error on rust side and use reject(...) to communicate the error
          resolve(data);
        };
      });

      this.zerdeEventloopEvents.callRust(name, params, callbackId);
      this.doWasmIo();
      return promise;
    };
    rpc.receive(WorkerEvent.CallRust, callRust);

    rpc.receive(WorkerEvent.CreateBuffer, (data: WrfArray) =>
      this.zerdeEventloopEvents.createWasmBuffer(data)
    );

    rpc.receive(WorkerEvent.CreateReadOnlyBuffer, (data: WrfArray) => {
      const bufferPtr = this.zerdeEventloopEvents.createWasmBuffer(data);
      const arcPtr = this.zerdeEventloopEvents.createArcVec(bufferPtr, data);
      return { bufferPtr, arcPtr };
    });

    rpc.receive(WorkerEvent.IncrementArc, (arcPtr: number) => {
      this.exports.incrementArc(BigInt(arcPtr));
    });

    rpc.receive(WorkerEvent.DecrementArc, (arcPtr: number) => {
      this.exports.decrementArc(BigInt(arcPtr));
    });

    rpc.receive(
      WorkerEvent.DeallocVec,
      ({ bufferPtr, bufferLen, bufferCap }: MutableBufferData) => {
        this.exports.deallocVec(
          BigInt(bufferPtr),
          BigInt(bufferLen),
          BigInt(bufferCap)
        );
      }
    );

    const bindMainWorkerPort = (port: MessagePort) => {
      const userWorkerRpc = new Rpc<Worker<WebWorkerRpc>>(port);
      userWorkerRpc.receive(MainWorkerChannelEvent.Init, () => ({
        wasmModule: this.module,
        memory: this.memory,
        taskWorkerSab,
        appPtr: this.appPtr,
        baseUri,
        tlsAndStackData: makeThreadLocalStorageAndStackDataOnExistingThread(
          this.exports
        ),
      }));
      userWorkerRpc.receive(
        MainWorkerChannelEvent.BindMainWorkerPort,
        (port: MessagePort) => {
          bindMainWorkerPort(port);
        }
      );

      userWorkerRpc.receive(MainWorkerChannelEvent.CallRust, callRust);

      userWorkerRpc.receive(
        MainWorkerChannelEvent.SendEventFromAnyThread,
        (eventPtr: BigInt) => {
          this.sendEventFromAnyThread(eventPtr);
        }
      );
    };
    rpc.receive(WorkerEvent.BindMainWorkerPort, (port) => {
      bindMainWorkerPort(port);
    });

    // create initial zerdeEventloopEvents
    this.zerdeEventloopEvents = new ZerdeEventloopEvents(this);

    // fetch dependencies
    this.zerdeEventloopEvents.fetchDeps();

    this.doWasmIo();

    this.doWasmBlock = true;

    // ok now, we wait for our resources to load.
    Promise.all(this.resources).then(this.doDepResults.bind(this));
  }

  private doDepResults(results: Resource[]): void {
    const deps: Dependency[] = [];
    // copy our reslts into wasm pointers
    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      // allocate pointer, do +8 because of the u64 length at the head of the buffer
      const vecLen = result.buffer.byteLength;
      const vecPtr = this.zerdeEventloopEvents.createWasmBuffer(
        new Uint8Array(result.buffer)
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
      width: this.sizingData.width,
      height: this.sizingData.height,
      dpiFactor: this.sizingData.dpiFactor,
      xrCanPresent: this.xrCanPresent,
      canFullscreen: this.sizingData.canFullscreen,
      xrIsPresenting: false,
    });
    this.doWasmBlock = false;
    this.doWasmIo();

    rpc.send(WorkerEvent.RemoveLoadingIndicators);
  }

  private doWasmIo(): void {
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

    // eslint-disable-next-line no-constant-condition
    while (true) {
      const msgType = this.zerdeParser.parseU32();
      if (this.sendFnTable[msgType](this)) {
        break;
      }
    }

    this.exports.deallocWasmMessage(BigInt(zerdeParserPtr));
  }

  // TODO(JP): Should use sychronous file loading for this.
  private loadDeps(deps: string[]): void {
    for (let i = 0; i < deps.length; i++) {
      const filePath = deps[i];
      this.resources.push(this.fetchPath(filePath));
    }
  }

  private setDocumentTitle(title: string): void {
    rpc.send(WorkerEvent.SetDocumentTitle, title);
  }

  private bindMouseAndTouch(): void {
    let lastMouseFinger;
    // TODO(JP): Some day bring back touch scroll support..
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

    const mouseFingers: {
      x: number;
      y: number;
      button: number;
      digit: number;
      time: number;
      modifiers: number;
      touch: boolean;
    }[] = [];
    function mouseToFinger(e: RpcMouseEvent | RpcWheelEvent): Finger {
      // @ts-ignore; TypeScript does not like the empty object declaration below, but we immediately fill every field
      const mf = mouseFingers[e.button] || (mouseFingers[e.button] = {});
      mf.x = e.pageX;
      mf.y = e.pageY;
      mf.button = e.button;
      mf.digit = e.button;
      mf.time = performance.now() / 1000.0;
      mf.modifiers = packKeyModifier(e);
      mf.touch = false;
      return mf;
    }

    const mouseButtonsDown: boolean[] = [];
    rpc.receive(WorkerEvent.CanvasMouseDown, (event: RpcMouseEvent) => {
      mouseButtonsDown[event.button] = true;
      this.zerdeEventloopEvents.fingerDown(mouseToFinger(event));
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowMouseUp, (event: RpcMouseEvent) => {
      mouseButtonsDown[event.button] = false;
      this.zerdeEventloopEvents.fingerUp(mouseToFinger(event));
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowMouseMove, (event: RpcMouseEvent) => {
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
    });
    rpc.receive(WorkerEvent.WindowMouseOut, (event: RpcMouseEvent) => {
      this.zerdeEventloopEvents.fingerOut(mouseToFinger(event));
      this.doWasmIo();
    });

    const touchIdsByDigit: (number | undefined)[] = [];
    rpc.receive(WorkerEvent.WindowTouchStart, (event: RpcTouchEvent) => {
      for (const touch of event.changedTouches) {
        let digit = touchIdsByDigit.indexOf(undefined);
        if (digit === -1) {
          digit = touchIdsByDigit.length;
        }
        touchIdsByDigit[digit] = touch.identifier;

        this.zerdeEventloopEvents.fingerDown({
          x: touch.pageX,
          y: touch.pageY,
          button: 0,
          digit,
          time: performance.now() / 1000.0,
          modifiers: packKeyModifier(event),
          touch: true,
        });
      }
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowTouchMove, (event: RpcTouchEvent) => {
      for (const touch of event.changedTouches) {
        const digit = touchIdsByDigit.indexOf(touch.identifier);
        if (digit == -1) {
          console.error("Unrecognized digit in WorkerEvent.WindowTouchMove");
          continue;
        }
        this.zerdeEventloopEvents.fingerMove({
          x: touch.pageX,
          y: touch.pageY,
          button: 0,
          digit,
          time: performance.now() / 1000.0,
          modifiers: packKeyModifier(event),
          touch: true,
        });
      }
      this.doWasmIo();
    });
    rpc.receive(
      WorkerEvent.WindowTouchEndCancelLeave,
      (event: RpcTouchEvent) => {
        for (const touch of event.changedTouches) {
          const digit = touchIdsByDigit.indexOf(touch.identifier);
          if (digit == -1) {
            console.error("Unrecognized digit in WorkerEvent.WindowTouchMove");
            continue;
          }
          touchIdsByDigit[digit] = undefined;
          this.zerdeEventloopEvents.fingerUp({
            x: touch.pageX,
            y: touch.pageY,
            button: 0,
            digit,
            time: performance.now() / 1000.0,
            modifiers: packKeyModifier(event),
            touch: true,
          });
        }
        this.doWasmIo();
      }
    );

    let lastWheelTime: number;
    let lastWasWheel: boolean;
    rpc.receive(WorkerEvent.CanvasWheel, (event: RpcWheelEvent) => {
      const finger = mouseToFinger(event);
      const delta = event.timeStamp - lastWheelTime;
      lastWheelTime = event.timeStamp;
      // typical web bullshit. this reliably detects mousewheel or touchpad on mac in safari
      if (isFirefox) {
        lastWasWheel = event.deltaMode == 1;
      } else {
        // detect it
        if (
          // @ts-ignore: TODO(Paras): wheelDeltaY looks different between browsers. Figure out a more consistent interface.
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

  private bindKeyboard(): void {
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

  private setMouseCursor(id: number): void {
    rpc.send(WorkerEvent.SetMouseCursor, cursorMap[id] || "default");
  }

  private startTimer(id: number, interval: number, repeats: number): void {
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

  private stopTimer(id: number): void {
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

  private httpSend(
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

  private websocketSend(url: string, data: Uint8Array): void {
    // TODO(Paras): Stop patching sendStack onto websockets
    // and maintain our own structure instead.
    const socket = this.websockets[url];
    if (!socket) {
      const socket = new WebSocket(url) as WebSocketWithSendStack;
      this.websockets[url] = socket;
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
        event.data.arrayBuffer().then((data: ArrayBuffer) => {
          this.zerdeEventloopEvents.websocketMessage(url, data);
          this.doWasmIo();
        });
      });
      socket.addEventListener("open", () => {
        const sendStack = socket.sendStack as Uint8Array[];
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

  private enableGlobalFileDropTarget(): void {
    rpc.send(WorkerEvent.EnableGlobalFileDropTarget);
    rpc.receive(WorkerEvent.DragEnter, () => {
      this.zerdeEventloopEvents.dragenter();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragOver, ({ x, y }: { x: number; y: number }) => {
      this.zerdeEventloopEvents.dragover(x, y);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragLeave, () => {
      this.zerdeEventloopEvents.dragleave();
      this.doWasmIo();
    });
    rpc.receive(
      WorkerEvent.Drop,
      ({
        fileHandles,
        fileHandlesToSend,
      }: {
        fileHandles: FileHandle[];
        fileHandlesToSend: FileHandle[];
      }) => {
        // We can't set this.fileHandles to a new object, since other places hold
        // references to it. Instead, clear it out and fill it up again.
        this.fileHandles.splice(0, this.fileHandles.length);
        this.fileHandles.push(...fileHandles);
        this.zerdeEventloopEvents.appOpenFiles(fileHandlesToSend);
        this.doWasmIo();
      }
    );
  }

  private async requestAnimationFrame(): Promise<void> {
    if (this.xrIsPresenting || this.hasRequestedAnimationFrame) {
      return;
    }
    this.hasRequestedAnimationFrame = true;
    if (this.runWebGLPromise) {
      await this.runWebGLPromise;
    }
    (self.requestAnimationFrame || self.setTimeout)(async () => {
      if (this.runWebGLPromise) {
        await this.runWebGLPromise;
      }
      this.hasRequestedAnimationFrame = false;
      if (this.xrIsPresenting) {
        return;
      }
      this.zerdeEventloopEvents.animationFrame();
      this.doWasmIo();
    });
  }

  // private runAsyncWebXRCheck(): void {
  //   this.xrCanPresent = false;
  //   this.xrIsPresenting = false;

  //   // ok this changes a bunch in how the renderflow works.
  //   // first thing we are going to do is get the vr displays.
  //   // @ts-ignore - Let's not worry about XR.
  //   const xrSystem = self.navigator.xr;
  //   if (xrSystem) {
  //     xrSystem.isSessionSupported("immersive-vr").then((supported) => {
  //       if (supported) {
  //         this.xrCanPresent = true;
  //       }
  //     });
  //   } else {
  //     console.log("No webVR support found");
  //   }
  // }

  private xrStartPresenting(): void {
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

  private xrStopPresenting(): void {
    // ignore for now
  }

  // TODO(JP): Should use sychronous file loading for this.
  private fetchPath(filePath: string): Promise<Resource> {
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

  sendEventFromAnyThread(eventPtr: BigInt): void {
    // Prevent an infinite loop when calling this from an event handler.
    setTimeout(() => {
      this.zerdeEventloopEvents.sendEventFromAnyThread(eventPtr);
      this.doWasmIo();
    });
  }

  // Array of function id's wasm can call on us; `self` is pointer to WasmApp.
  // Function names are suffixed with the index in the array, and annotated with
  // their name in cx_wasm32.rs, for easier matching.
  private sendFnTable: ((self: this) => void | boolean)[] = [
    // end
    function end0(_self) {
      return true;
    },
    // run_webgl
    function runWebGL1(self) {
      const zerdeParserPtr = self.zerdeParser.parseU64();
      if (self.webglRenderer) {
        self.webglRenderer.processMessages(Number(zerdeParserPtr));
        self.exports.deallocWasmMessage(zerdeParserPtr);
      } else {
        self.runWebGLPromise = rpc
          .send(WorkerEvent.RunWebGL, Number(zerdeParserPtr))
          .then(() => {
            self.exports.deallocWasmMessage(zerdeParserPtr);
            self.runWebGLPromise = undefined;
          });
      }
    },
    // log
    function log2(self) {
      console.log(self.zerdeParser.parseString());
    },
    // load_deps
    function loadDeps3(self) {
      const deps: string[] = [];
      const numDeps = self.zerdeParser.parseU32();
      for (let i = 0; i < numDeps; i++) {
        deps.push(self.zerdeParser.parseString());
      }
      self.loadDeps(deps);
    },
    // request_animation_frame
    function requestAnimationFrame4(self) {
      self.requestAnimationFrame();
    },
    // set_document_title
    function setDocumentTitle5(self) {
      self.setDocumentTitle(self.zerdeParser.parseString());
    },
    // set_mouse_cursor
    function setMouseCursor6(self) {
      self.setMouseCursor(self.zerdeParser.parseU32());
    },
    // show_text_ime
    function showTextIme7(self) {
      const x = self.zerdeParser.parseF32();
      const y = self.zerdeParser.parseF32();
      rpc.send(WorkerEvent.ShowTextIME, { x, y });
    },
    // hide_text_ime
    function hideTextIme8(_self) {
      // TODO(JP): doesn't seem to do anything, is that intentional?
    },
    // text_copy_response
    function textCopyResponse9(self) {
      const textCopyResponse = self.zerdeParser.parseString();
      rpc.send(WorkerEvent.TextCopyResponse, textCopyResponse);
    },
    // start_timer
    function startTimer10(self) {
      const repeats = self.zerdeParser.parseU32();
      const id = self.zerdeParser.parseF64();
      const interval = self.zerdeParser.parseF64();
      self.startTimer(id, interval, repeats);
    },
    // stop_timer
    function stopTimer11(self) {
      const id = self.zerdeParser.parseF64();
      self.stopTimer(id);
    },
    // xr_start_presenting
    function xrStartPresenting12(self) {
      self.xrStartPresenting();
    },
    // xr_stop_presenting
    function xrStopPresenting13(self) {
      self.xrStopPresenting();
    },
    // http_send
    function httpSend14(self) {
      const port = self.zerdeParser.parseU32();
      const signalId = self.zerdeParser.parseU32();
      const verb = self.zerdeParser.parseString();
      const path = self.zerdeParser.parseString();
      const proto = self.zerdeParser.parseString();
      const domain = self.zerdeParser.parseString();
      const contentType = self.zerdeParser.parseString();
      const body = self.zerdeParser.parseU8Slice();
      self.httpSend(
        verb,
        path,
        proto,
        domain,
        port,
        contentType,
        body,
        signalId
      );
    },
    // fullscreen
    function fullscreen15(_self) {
      rpc.send(WorkerEvent.Fullscreen);
    },
    // normalscreen
    function normalscreen16(_self) {
      rpc.send(WorkerEvent.Normalscreen);
    },
    // websocket_send
    function websocketSend17(self) {
      const url = self.zerdeParser.parseString();
      const data = self.zerdeParser.parseU8Slice();
      self.websocketSend(url, data);
    },
    // enable_global_file_drop_target
    function enableGlobalFileDropTarget18(self) {
      self.enableGlobalFileDropTarget();
    },
    // call_js
    function callJs19(self) {
      const fnName = self.zerdeParser.parseString();
      const params = self.zerdeParser.parseWrfParams();
      if (fnName === "_wrflibReturnParams") {
        const callbackId = JSON.parse(params[0] as string);
        self.callRustPendingCallbacks[callbackId](params.slice(1));
        delete self.callRustPendingCallbacks[callbackId];
      } else {
        rpc.send(WorkerEvent.CallJs, { fnName, params });
      }
    },
  ];
}

rpc.receive(
  WorkerEvent.Init,
  ({
    wasmModule,
    offscreenCanvas,
    sizingData,
    baseUri,
    memory,
    taskWorkerSab,
  }) => {
    let wasmapp: WasmApp;
    return new Promise<void>((resolve, reject) => {
      const threadSpawn = (ctxPtr: BigInt) => {
        rpc.send(WorkerEvent.ThreadSpawn, {
          ctxPtr,
          tlsAndStackData: makeThreadLocalStorageAndStackDataOnExistingThread(
            wasmapp.exports
          ),
        });
      };

      const getExports = () => {
        return wasmapp.exports;
      };

      const fileHandles: FileHandle[] = [];

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

      WebAssembly.instantiate(wasmModule, { env }).then((instance: any) => {
        const wasmExports = instance.exports as WasmExports;
        initThreadLocalStorageMainWorker(wasmExports);
        wasmapp = new WasmApp({
          offscreenCanvas,
          wasmModule,
          wasmExports,
          memory,
          sizingData,
          baseUri,
          fileHandles,
          taskWorkerSab,
        });
        resolve();
      }, reject);
    });
  }
);
