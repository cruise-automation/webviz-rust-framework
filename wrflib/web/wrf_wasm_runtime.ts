import {
  getCachedUint8Buffer,
  getWrfBufferWasm,
  isWrfBuffer,
  overwriteTypedArraysWithWrfArrays,
  unregisterMutableBuffer,
  WrfBuffer,
  wrfArrayCoversWrfBuffer,
} from "./wrf_buffer";
import { Rpc } from "./common";
import { makeRpcEvent } from "./make_rpc_event";
import { makeTextarea, TextareaEvent } from "./make_textarea";
import {
  BufferData,
  CallJSData,
  CallRust,
  CallJsCallback,
  PostMessageTypedArray,
  CallRustInSameThreadSync,
  WorkerEvent,
  SizingData,
} from "./types";

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

let rpc: Rpc;

export const wrfNewWorkerPort = (): MessagePort => {
  const channel = new MessageChannel();
  rpc.send(WorkerEvent.BindUserWorkerPortOnMainThread, channel.port1, [
    channel.port1,
  ]);
  return channel.port2;
};

let wasmMemory: WebAssembly.Memory;

const destructor = (arcPtr: number) => {
  rpc.send(WorkerEvent.DecrementArc, arcPtr);
};

const mutableDestructor = (bufferData: BufferData) => {
  rpc.send(WorkerEvent.DeallocVec, bufferData);
};

function transformParamsFromRust(params: (string | BufferData)[]) {
  return params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const wrfBuffer = getWrfBufferWasm(
        wasmMemory,
        param,
        destructor,
        mutableDestructor
      );
      return getCachedUint8Buffer(
        wrfBuffer,
        // This actually creates a WrfUint8Array as this was overwritten above in overwriteTypedArraysWithWrfArrays()
        new Uint8Array(wrfBuffer, param.bufferPtr, param.bufferLen)
      );
    }
  });
}

// TODO(JP): Somewhat duplicated with the other implementation.
const temporarilyHeldBuffersForPostMessage = new Set();
export const serializeWrfArrayForPostMessage = (
  wrfArray: Uint8Array
): PostMessageTypedArray => {
  if (!(typeof wrfArray === "object" && isWrfBuffer(wrfArray.buffer))) {
    throw new Error("Only pass Wrf arrays to serializeWrfArrayForPostMessage");
  }
  const wrfBuffer = wrfArray.buffer as WrfBuffer;

  if (wrfBuffer.readonly) {
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
      if (!wrfArrayCoversWrfBuffer(param)) {
        throw new Error(
          "callRust only supports buffers that span the entire underlying WrfBuffer"
        );
      }
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

export const createBuffer = async (data: Uint8Array): Promise<Uint8Array> => {
  const bufferLen = data.byteLength;
  const bufferPtr = await rpc.send<number>(WorkerEvent.CreateBuffer, data, [
    data.buffer,
  ]);

  return transformParamsFromRust([
    {
      bufferPtr,
      bufferLen,
      bufferCap: bufferLen,
      arcPtr: null,
    },
  ])[0] as Uint8Array;
};

export const createReadOnlyBuffer = async (
  data: Uint8Array
): Promise<Uint8Array> => {
  const bufferLen = data.byteLength;
  const { bufferPtr, arcPtr } = await rpc.send<{
    bufferPtr: number;
    arcPtr: number;
  }>(WorkerEvent.CreateReadOnlyBuffer, data, [data.buffer]);

  return transformParamsFromRust([
    {
      bufferPtr,
      bufferLen,
      bufferCap: undefined,
      arcPtr,
    },
  ])[0] as Uint8Array;
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

    const loader = () => {
      const isMobileSafari = self.navigator.platform.match(/iPhone|iPad/i);
      const isAndroid = self.navigator.userAgent.match(/Android/i);

      let rpcInitialized = false;

      rpc.receive(WorkerEvent.ShowIncompatibleBrowserNotification, () => {
        const span = document.createElement("span");
        span.style.color = "white";
        canvas.parentNode.replaceChild(span, canvas);
        span.innerHTML =
          "Sorry, we need browser support for WebGL to run<br/>Please update your browser to a more modern one<br/>Update to at least iOS 10, Safari 10, latest Chrome, Edge or Firefox<br/>Go and update and come back, your browser will be better, faster and more secure!<br/>If you are using chrome on OSX on a 2011/2012 mac please enable your GPU at: Override software rendering list:Enable (the top item) in: <a href='about://flags'>about://flags</a>. Or switch to Firefox or Safari.";
      });

      rpc.receive(WorkerEvent.RemoveLoadingIndicators, () => {
        const loaders = document.getElementsByClassName("cx_webgl_loader");
        for (let i = 0; i < loaders.length; i++) {
          loaders[i].parentNode.removeChild(loaders[i]);
        }
      });

      rpc.receive(WorkerEvent.SetDocumentTitle, ({ title }) => {
        document.title = title;
      });

      rpc.receive(WorkerEvent.SetMouseCursor, ({ style }) => {
        document.body.style.cursor = style;
      });

      rpc.receive(WorkerEvent.Fullscreen, () => {
        if (document.body.requestFullscreen) {
          document.body.requestFullscreen();
        } else if (document.body["webkitRequestFullscreen"]) {
          document.body["webkitRequestFullscreen"]();
        } else if (document.body["mozRequestFullscreen"]) {
          document.body["mozRequestFullscreen"]();
        }
      });

      rpc.receive(WorkerEvent.Normalscreen, () => {
        if (document.exitFullscreen) {
          document.exitFullscreen();
        } else if (document["webkitExitFullscreen"]) {
          document["webkitExitFullscreen"]();
        } else if (document["mozExitFullscreen"]) {
          document["mozExitFullscreen"]();
        }
      });

      rpc.receive(WorkerEvent.TextCopyResponse, ({ textCopyResponse }) => {
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
          if (rpcInitialized) rpc.send(WorkerEvent.Drop, { files });
        });
      });

      rpc.receive(WorkerEvent.CallJs, ({ fnName, params }: CallJSData) => {
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
          rpc.send(WorkerEvent.CanvasMouseDown, {
            event: makeRpcEvent(event),
          });
      });
      window.addEventListener("mouseup", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseUp, {
            event: makeRpcEvent(event),
          });
      });
      window.addEventListener("mousemove", (event) => {
        document.body.scrollTop = 0;
        document.body.scrollLeft = 0;
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseMove, {
            event: makeRpcEvent(event),
          });
      });
      window.addEventListener("mouseout", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.WindowMouseOut, {
            event: makeRpcEvent(event),
          });
      });
      document.addEventListener("wheel", (event) => {
        if (rpcInitialized)
          rpc.send(WorkerEvent.CanvasWheel, {
            event: makeRpcEvent(event),
          });
      });
      window.addEventListener("focus", () => {
        if (rpcInitialized) rpc.send(WorkerEvent.WindowFocus, {});
      });
      window.addEventListener("blur", () => {
        if (rpcInitialized) rpc.send(WorkerEvent.WindowBlur, {});
      });

      if (!isMobileSafari && !isAndroid) {
        // mobile keyboards are unusable on a UI like this
        const { showTextIME } = makeTextarea((taEvent: TextareaEvent) => {
          const eventType: WorkerEvent = taEvent.type;
          if (rpcInitialized) rpc.send(eventType, taEvent);
        });
        rpc.receive(WorkerEvent.ShowTextIME, showTextIME);
      }

      function getSizingData(): SizingData {
        const canFullscreen = !!(
          document.fullscreenEnabled ||
          document["webkitFullscreenEnabled"] ||
          document["mozFullscreenEnabled"]
        );
        const isFullscreen = !!(
          document.fullscreenElement ||
          document["webkitFullscreenElement"] ||
          document["mozFullscreenElement"]
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

        if (rpcInitialized) rpc.send(WorkerEvent.ScreenResize, getSizingData());
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
        self.setInterval((_) => {
          if (window.devicePixelRatio != dpiFactor) {
            dpiFactor = window.devicePixelRatio;
            onScreenResize();
          }
        }, 1000);
      }

      const offscreenCanvas = canvas.transferControlToOffscreen();
      const baseUri = document.baseURI;

      // Initial has to be equal to or higher than required by the app (which at the time of writing
      // is around 20 pages).
      // Maximum has to be equal to or lower than that of the app, which we've currently set to
      // the maximum for wasm32 (4GB). Browsers should use virtual memory, as to not actually take up
      // all this space until requested by the app. TODO(JP): We might need to check this behavior in
      // different browsers at some point (in Chrome it seems to work fine).
      wasmMemory = new WebAssembly.Memory({
        initial: 40,
        maximum: 65535,
        shared: true,
      });

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

      rpc
        .send(
          WorkerEvent.Init,
          {
            offscreenCanvas,
            wasmFilename,
            sizingData: getSizingData(),
            baseUri,
            memory: wasmMemory,
          },
          [offscreenCanvas]
        )
        .then(() => {
          rpcInitialized = true;
          onScreenResize();
          resolve();
        });
    };

    if (document.readyState !== "loading") {
      loader();
    } else {
      document.addEventListener("DOMContentLoaded", loader);
    }
  });
