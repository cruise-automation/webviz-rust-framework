import {
  getWrfBufferWasm,
  isWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  unregisterMutableBuffer,
  WrfBuffer,
  checkValidWrfArray,
} from "./wrf_buffer";
import {
  assertNotNull,
  getWrfParamType,
  initTaskWorkerSab,
  Rpc,
  transformParamsFromRustImpl,
} from "./common";
import { makeTextarea, TextareaEvent } from "./make_textarea";
import {
  CallRust,
  CallJsCallback,
  PostMessageTypedArray,
  CallRustInSameThreadSync,
  SizingData,
  TlsAndStackData,
  WrfArray,
  CreateBuffer,
  FileHandle,
  MutableBufferData,
  RustWrfParam,
} from "./types";
import { WebGLRenderer } from "./webgl_renderer";
import {
  makeRpcMouseEvent,
  makeRpcTouchEvent,
  makeRpcWheelEvent,
} from "./make_rpc_event";
import {
  AsyncWorkerRpc,
  WasmWorkerRpc,
  WorkerEvent,
  TaskWorkerEvent,
  AsyncWorkerEvent,
} from "./rpc_types";

declare global {
  interface Document {
    ExitFullscreen?: () => Promise<void>;
    webkitExitFullscreen?: () => Promise<void>;
    mozExitFullscreen?: () => Promise<void>;
    webkitFullscreenEnabled?: () => Promise<void>;
    mozFullscreenEnabled?: () => Promise<void>;
    webkitFullscreenElement?: () => Promise<void>;
    mozFullscreenElement?: () => Promise<void>;
  }
  interface HTMLElement {
    mozRequestFullscreen?: () => Promise<void>;
    webkitRequestFullscreen?: () => Promise<void>;
  }
}

const jsFunctions: Record<string, CallJsCallback> = {};

/// Users must call this function to register functions as runnable from
/// Rust via `[Cx::call_js]`.
export const registerCallJsCallbacks = (
  fns: Record<string, CallJsCallback>
): void => {
  // Check that all new functions are unique
  for (const key of Object.keys(fns)) {
    if (key in jsFunctions) {
      throw new Error(
        `Error: overwriting existing function "${key}" in window.jsFunctions`
      );
    }
  }

  Object.assign(jsFunctions, fns);
};
/// Users must call this function to unregister functions as runnable from
/// Rust via `[Cx::call_js]`.
export const unregisterCallJsCallbacks = (fnNames: string[]): void => {
  for (const name of fnNames) {
    // Check that functions are registered
    if (!(name in jsFunctions)) {
      throw new Error(`Error: unregistering non-existent function "${name}".`);
    }

    delete jsFunctions[name];
  }
};

let rpc: Rpc<WasmWorkerRpc>;

export const wrfNewWorkerPort = (): MessagePort => {
  const channel = new MessageChannel();
  rpc.send(WorkerEvent.BindMainWorkerPort, channel.port1, [channel.port1]);
  return channel.port2;
};

let wasmMemory: WebAssembly.Memory;

const destructor = (arcPtr: number) => {
  rpc.send(WorkerEvent.DecrementArc, arcPtr);
};

const mutableDestructor = (bufferData: MutableBufferData) => {
  rpc.send(WorkerEvent.DeallocVec, bufferData);
};

const transformParamsFromRust = (params: RustWrfParam[]) =>
  transformParamsFromRustImpl(
    wasmMemory,
    destructor,
    mutableDestructor,
    params
  );

// TODO(JP): Somewhat duplicated with the other implementation.
const temporarilyHeldBuffersForPostMessage = new Set();
export const serializeWrfArrayForPostMessage = (
  wrfArray: WrfArray
): PostMessageTypedArray => {
  if (!(typeof wrfArray === "object" && isWrfBuffer(wrfArray.buffer))) {
    throw new Error("Only pass Wrf arrays to serializeWrfArrayForPostMessage");
  }
  const wrfBuffer = wrfArray.buffer as WrfBuffer;

  if (wrfBuffer.__wrflibBufferData.readonly) {
    // Store the buffer temporarily until we've received confirmation that the Arc has been incremented.
    // Otherwise it might get garbage collected and deallocated (if the Arc's count was 1) before it gets
    // incremented.
    temporarilyHeldBuffersForPostMessage.add(wrfBuffer);
    rpc
      .send(WorkerEvent.IncrementArc, wrfBuffer.__wrflibBufferData.arcPtr)
      .then(() => {
        temporarilyHeldBuffersForPostMessage.delete(wrfBuffer);
      });
  } else {
    unregisterMutableBuffer(wrfBuffer);
  }

  return {
    bufferData: wrfBuffer.__wrflibBufferData,
    byteOffset: wrfArray.byteOffset,
    byteLength: wrfArray.byteLength,
  };
};

export const callRust: CallRust = async (name, params = []) => {
  const transformedParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else if (isWrfBuffer(param.buffer)) {
      checkValidWrfArray(param);
      return serializeWrfArrayForPostMessage(param);
    } else {
      if (!(param.buffer instanceof SharedArrayBuffer)) {
        console.warn(
          "Consider passing Uint8Arrays backed by WrfBuffer or SharedArrayBuffer into `callRust` to prevent copying data"
        );
      }
      return param;
    }
  });

  return transformParamsFromRust(
    await rpc.send(WorkerEvent.CallRust, { name, params: transformedParams })
  );
};

export const createBuffer: CreateBuffer = async (data) => {
  const bufferLen = data.byteLength;
  const bufferPtr = await rpc.send(WorkerEvent.CreateBuffer, data, [
    data.buffer,
  ]);

  return transformParamsFromRust([
    {
      paramType: getWrfParamType(data, false),
      bufferPtr,
      bufferLen,
      bufferCap: bufferLen,
      readonly: false,
    },
  ])[0] as typeof data;
};

export const createReadOnlyBuffer: CreateBuffer = async (data) => {
  const bufferLen = data.byteLength;
  const { bufferPtr, arcPtr } = await rpc.send(
    WorkerEvent.CreateReadOnlyBuffer,
    data,
    [data.buffer]
  );

  return transformParamsFromRust([
    {
      paramType: getWrfParamType(data, true),
      bufferPtr,
      bufferLen,
      arcPtr,
      readonly: true,
    },
  ])[0] as typeof data;
};

export const deserializeWrfArrayFromPostMessage = (
  postMessageData: PostMessageTypedArray
): Uint8Array => {
  const wrfBuffer = getWrfBufferWasm(
    wasmMemory,
    postMessageData.bufferData,
    destructor,
    mutableDestructor
  );
  return new Uint8Array(
    wrfBuffer,
    postMessageData.byteOffset,
    postMessageData.byteLength
  );
};

export const callRustInSameThreadSync: CallRustInSameThreadSync = (
  name,
  _params = []
) => {
  throw new Error(
    "`callRustInSameThreadSync` is currently not supported on the main thread in WASM"
  );
};

export const initialize = (
  initParams: { filename: string } | { targetName: string }
): Promise<void> =>
  new Promise<void>((resolve) => {
    overwriteTypedArraysWithWrfArrays();

    rpc = new Rpc(new Worker(new URL("./wrf_wasm_worker.ts", import.meta.url)));

    let wasmFilename: string;
    if ("filename" in initParams) {
      wasmFilename = initParams.filename;
    } else {
      // @ts-ignore
      const env = new URL(window.document.location).searchParams.get("debug")
        ? "debug"
        : "release";
      wasmFilename = `target/wasm32-unknown-unknown/${env}/${initParams.targetName}.wasm`;
    }
    const wasmPath = new URL(wasmFilename, document.baseURI).href;

    // Safari (as of version 15.2) needs the WebAssembly Module to be compiled on the browser's
    // main thread. This also allows us to start compiling while still waiting for the DOM to load.
    const wasmModulePromise = WebAssembly.compileStreaming(fetch(wasmPath));

    // TODO(JP): These file handles are only sent to a worker when it starts running;
    // it currently can't receive any file handles added after that.
    const fileHandles: FileHandle[] = [];

    const loader = () => {
      const isMobileSafari = self.navigator.platform.match(/iPhone|iPad/i);
      const isAndroid = self.navigator.userAgent.match(/Android/i);

      let rpcInitialized = false;

      rpc.receive(WorkerEvent.ShowIncompatibleBrowserNotification, () => {
        const span = document.createElement("span");
        span.style.color = "white";
        assertNotNull(canvas.parentNode).replaceChild(span, canvas);
        span.innerHTML =
          "Sorry, we need browser support for WebGL to run<br/>Please update your browser to a more modern one<br/>Update to at least iOS 10, Safari 10, latest Chrome, Edge or Firefox<br/>Go and update and come back, your browser will be better, faster and more secure!<br/>If you are using chrome on OSX on a 2011/2012 mac please enable your GPU at: Override software rendering list:Enable (the top item) in: <a href='about://flags'>about://flags</a>. Or switch to Firefox or Safari.";
      });

      rpc.receive(WorkerEvent.RemoveLoadingIndicators, () => {
        const loaders = document.getElementsByClassName("cx_webgl_loader");
        for (let i = 0; i < loaders.length; i++) {
          assertNotNull(loaders[i].parentNode).removeChild(loaders[i]);
        }
      });

      rpc.receive(WorkerEvent.SetDocumentTitle, (title: string) => {
        document.title = title;
      });

      rpc.receive(WorkerEvent.SetMouseCursor, (style: string) => {
        document.body.style.cursor = style;
      });

      rpc.receive(WorkerEvent.Fullscreen, () => {
        if (document.body.requestFullscreen) {
          document.body.requestFullscreen();
        } else if (document.body.webkitRequestFullscreen) {
          document.body.webkitRequestFullscreen();
        } else if (document.body.mozRequestFullscreen) {
          document.body.mozRequestFullscreen();
        }
      });

      rpc.receive(WorkerEvent.Normalscreen, () => {
        if (document.exitFullscreen) {
          document.exitFullscreen();
        } else if (document.webkitExitFullscreen) {
          document.webkitExitFullscreen();
        } else if (document.mozExitFullscreen) {
          document.mozExitFullscreen();
        }
      });

      rpc.receive(WorkerEvent.TextCopyResponse, (textCopyResponse: string) => {
        window.navigator.clipboard.writeText(textCopyResponse);
      });

      rpc.receive(WorkerEvent.EnableGlobalFileDropTarget, () => {
        document.addEventListener("dragenter", (ev) => {
          const dataTransfer = ev.dataTransfer;
          // dataTransfer isn't guaranteed to exist by spec, so it must be checked
          if (
            dataTransfer &&
            dataTransfer.types.length === 1 &&
            dataTransfer.types[0] === "Files"
          ) {
            ev.stopPropagation();
            ev.preventDefault();
            dataTransfer.dropEffect = "copy";
            if (rpcInitialized) rpc.send(WorkerEvent.DragEnter);
          }
        });
        document.addEventListener("dragover", (ev) => {
          ev.stopPropagation();
          ev.preventDefault();
          if (rpcInitialized)
            rpc.send(WorkerEvent.DragOver, { x: ev.clientX, y: ev.clientY });
        });
        document.addEventListener("dragleave", (ev) => {
          ev.stopPropagation();
          ev.preventDefault();
          if (rpcInitialized) rpc.send(WorkerEvent.DragLeave);
        });
        document.addEventListener("drop", (ev) => {
          if (!ev.dataTransfer) {
            return;
          }
          const files = Array.from(ev.dataTransfer.files);
          if (!files.length) {
            return;
          }
          ev.preventDefault();
          ev.stopPropagation();
          const fileHandlesToSend: FileHandle[] = [];
          for (const file of files) {
            const fileHandle = {
              id: fileHandles.length,
              basename: file.name,
              file,
              lastReadStart: -1,
              lastReadEnd: -1,
            };
            fileHandlesToSend.push(fileHandle);
            fileHandles.push(fileHandle);
          }
          if (rpcInitialized) {
            rpc.send(WorkerEvent.Drop, { fileHandles, fileHandlesToSend });
          }
        });
      });

      rpc.receive(WorkerEvent.CallJs, ({ fnName, params }) => {
        const fn = jsFunctions[fnName];
        if (!fn) {
          console.error(
            `call_js with ${fnName} is not available. Have you registered it using \`registerCallJsCallbacks\`?`
          );
          return;
        }

        fn(transformParamsFromRust(params));
      });

      const canvas: HTMLCanvasElement = document.createElement("canvas");
      canvas.className = "cx_webgl";
      document.body.appendChild(canvas);

      const loadingIndicator = document.createElement("div");
      loadingIndicator.className = "cx_webgl_loader";
      loadingIndicator.innerHTML =
        '<span>⚡</span><div style="color: rgba(255, 202, 0, 0.5);">Loading…</div>';
      document.body.appendChild(loadingIndicator);

      document.addEventListener("contextmenu", (event) => {
        if (
          event.target instanceof Element &&
          (!document.getElementById("js_root")?.contains(event.target) ||
            Array.from(event.target.classList).includes("wrflibPanel"))
        ) {
          event.preventDefault();
        }
      });

      document.addEventListener("mousedown", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.CanvasMouseDown, makeRpcMouseEvent(event));
      });
      window.addEventListener("mouseup", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseUp, makeRpcMouseEvent(event));
      });
      window.addEventListener("mousemove", (event) => {
        document.body.scrollTop = 0;
        document.body.scrollLeft = 0;
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseMove, makeRpcMouseEvent(event));
      });
      window.addEventListener("mouseout", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseOut, makeRpcMouseEvent(event));
      });

      document.addEventListener(
        "touchstart",
        (event: TouchEvent) => {
          event.preventDefault();
          if (rpcInitialized)
            rpc.send(WorkerEvent.WindowTouchStart, makeRpcTouchEvent(event));
        },
        { passive: false }
      );
      window.addEventListener(
        "touchmove",
        (event: TouchEvent) => {
          event.preventDefault();
          if (rpcInitialized)
            rpc.send(WorkerEvent.WindowTouchMove, makeRpcTouchEvent(event));
        },
        { passive: false }
      );
      const touchEndCancelLeave = (event: TouchEvent) => {
        event.preventDefault();
        if (rpcInitialized)
          rpc.send(
            WorkerEvent.WindowTouchEndCancelLeave,
            makeRpcTouchEvent(event)
          );
      };
      window.addEventListener("touchend", touchEndCancelLeave);
      window.addEventListener("touchcancel", touchEndCancelLeave);

      document.addEventListener("wheel", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.CanvasWheel, makeRpcWheelEvent(event));
      });
      window.addEventListener("focus", () => {
        if (rpcInitialized) rpc.send(WorkerEvent.WindowFocus);
      });
      window.addEventListener("blur", () => {
        if (rpcInitialized) rpc.send(WorkerEvent.WindowBlur);
      });

      if (!isMobileSafari && !isAndroid) {
        // mobile keyboards are unusable on a UI like this
        const { showTextIME } = makeTextarea((taEvent: TextareaEvent) => {
          if (rpcInitialized) rpc.send(taEvent.type, taEvent);
        });
        rpc.receive(WorkerEvent.ShowTextIME, showTextIME);
      }

      // One of these variables should get set, depending on if
      // the browser supports OffscreenCanvas or not.
      let offscreenCanvas: OffscreenCanvas;
      let webglRenderer: WebGLRenderer;

      function getSizingData(): SizingData {
        const canFullscreen = !!(
          document.fullscreenEnabled ||
          document.webkitFullscreenEnabled ||
          document.mozFullscreenEnabled
        );
        const isFullscreen = !!(
          document.fullscreenElement ||
          document.webkitFullscreenElement ||
          document.mozFullscreenElement
        );
        return {
          width: canvas.offsetWidth,
          height: canvas.offsetHeight,
          dpiFactor: window.devicePixelRatio,
          canFullscreen,
          isFullscreen,
        };
      }

      function onScreenResize() {
        // TODO(JP): Some day bring this back?
        // if (is_add_to_homescreen_safari) { // extremely ugly. but whatever.
        //     if (window.orientation == 90 || window.orientation == -90) {
        //         h = screen.width;
        //         w = screen.height - 90;
        //     }
        //     else {
        //         w = screen.width;
        //         h = screen.height - 80;
        //     }
        // }

        const sizingData = getSizingData();
        if (webglRenderer) {
          webglRenderer.resize(sizingData);
        }
        if (rpcInitialized) rpc.send(WorkerEvent.ScreenResize, sizingData);
      }
      window.addEventListener("resize", () => onScreenResize());
      window.addEventListener("orientationchange", () => onScreenResize());

      let dpiFactor = window.devicePixelRatio;
      const mqString = "(resolution: " + window.devicePixelRatio + "dppx)";
      const mq = matchMedia(mqString);
      if (mq && mq.addEventListener) {
        mq.addEventListener("change", () => onScreenResize());
      } else {
        // poll for it. yes. its terrible
        self.setInterval(() => {
          if (window.devicePixelRatio != dpiFactor) {
            dpiFactor = window.devicePixelRatio;
            onScreenResize();
          }
        }, 1000);
      }

      // Some browsers (e.g. Safari 15.2) require SharedArrayBuffers to be initialized
      // on the browser's main thread; so that's why this has to happen here.
      //
      // We also do this before initializing `WebAssembly.Memory`, to make sure we have
      // enough memory for both.. (This is mostly relevant on mobile; see note below.)
      const taskWorkerSab = initTaskWorkerSab();
      const taskWorkerRpc = new Rpc(
        new Worker(new URL("./task_worker.ts", import.meta.url))
      );
      taskWorkerRpc.send(TaskWorkerEvent.Init, {
        taskWorkerSab,
        wasmMemory,
      });

      // Initial has to be equal to or higher than required by the app (which at the time of writing
      // is around 20 pages).
      // Maximum has to be equal to or lower than that of the app, which we've currently set to
      // the maximum for wasm32 (4GB). Browsers should use virtual memory, as to not actually take up
      // all this space until requested by the app. TODO(JP): We might need to check this behavior in
      // different browsers at some point (in Chrome it seems to work fine).
      //
      // In Safari on my phone (JP), using maximum:65535 causes an out-of-memory error, so we instead
      // try a hardcoded value of ~400MB.. Note that especially on mobile, all of
      // this is quite tricky; see e.g. https://github.com/WebAssembly/design/issues/1397
      //
      // TODO(JP): It looks like when using shared memory, the maximum might get fully allocated on
      // some devices (mobile?), which means that there is little room left for JS objects, and it
      // means that the web page is at higher risk of getting evicted when switching tabs. There are a
      // few options here:
      // 1. Allow the user to specify a maximum by hand for mobile in general; or for specific
      //    devices (cumbersome!).
      // 2. Allow single-threaded operation, where we don't specify a maximum (but run the risk of
      //    getting much less memory to use and therefore the app crashing; see again
      //    https://github.com/WebAssembly/design/issues/1397 for more details).
      try {
        wasmMemory = new WebAssembly.Memory({
          initial: 40,
          maximum: 65535,
          shared: true,
        });
      } catch (_) {
        console.log("Can't allocate full WebAssembly memory; trying ~400MB");
        try {
          wasmMemory = new WebAssembly.Memory({
            initial: 40,
            maximum: 6000,
            shared: true,
          });
        } catch (_) {
          throw new Error("Can't initilialize WebAssembly memory..");
        }
      }

      // If the browser supports OffscreenCanvas, then we'll use that. Otherwise, we render on
      // the browser's main thread using WebGLRenderer.
      try {
        offscreenCanvas = canvas.transferControlToOffscreen();
      } catch (_) {
        webglRenderer = new WebGLRenderer(
          canvas,
          wasmMemory,
          getSizingData(),
          () => {
            rpc.send(WorkerEvent.ShowIncompatibleBrowserNotification);
          }
        );
        rpc.receive(WorkerEvent.RunWebGL, (zerdeParserPtr) => {
          webglRenderer.processMessages(zerdeParserPtr);
          return new Promise((resolve) => {
            requestAnimationFrame(() => {
              resolve(undefined);
            });
          });
        });
      }

      wasmModulePromise.then((wasmModule) => {
        // Threads need to be spawned on the browser's main thread, otherwise Safari (as of version 15.2)
        // throws errors.
        const asyncWorkers = new Set();
        const threadSpawn = ({
          ctxPtr,
          tlsAndStackData,
        }: {
          ctxPtr: BigInt;
          tlsAndStackData: TlsAndStackData;
        }) => {
          const worker = new Worker(
            new URL("./async_worker.ts", import.meta.url)
          );
          const workerErrorHandler = (event: unknown) => {
            console.log("Async worker error event: ", event);
          };
          worker.onerror = workerErrorHandler;
          worker.onmessageerror = workerErrorHandler;
          const workerRpc = new Rpc<AsyncWorkerRpc>(worker);

          // Add the worker to an array of workers, to prevent them getting killed when
          // during garbage collection in Firefox; see https://bugzilla.mozilla.org/show_bug.cgi?id=1592227
          asyncWorkers.add(worker);

          const channel = new MessageChannel();
          rpc.send(WorkerEvent.BindMainWorkerPort, channel.port1, [
            channel.port1,
          ]);

          workerRpc.receive(AsyncWorkerEvent.ThreadSpawn, threadSpawn);

          workerRpc
            .send(
              AsyncWorkerEvent.Run,
              {
                wasmModule,
                memory: wasmMemory,
                taskWorkerSab,
                ctxPtr,
                fileHandles,
                baseUri: document.baseURI,
                tlsAndStackData,
                mainWorkerPort: channel.port2,
              },
              [channel.port2]
            )
            .catch((e) => {
              console.error("async worker failed", e);
            })
            .finally(() => {
              worker.terminate();
              asyncWorkers.delete(worker);
            });
        };
        rpc.receive(WorkerEvent.ThreadSpawn, threadSpawn);

        rpc
          .send(
            WorkerEvent.Init,
            {
              wasmModule,
              offscreenCanvas,
              sizingData: getSizingData(),
              baseUri: document.baseURI,
              memory: wasmMemory,
              taskWorkerSab,
            },
            offscreenCanvas ? [offscreenCanvas] : []
          )
          .then(() => {
            rpcInitialized = true;
            onScreenResize();
            resolve();
          });
      });
    };

    if (document.readyState !== "loading") {
      loader();
    } else {
      document.addEventListener("DOMContentLoaded", loader);
    }
  });
