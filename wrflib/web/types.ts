// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

export type Uniform = { ty: string; name: string };

export type ShaderAttributes = {
  shaderId: number;
  fragment: string;
  vertex: string;
  geometrySlots: number;
  instanceSlots: number;
  passUniforms: Uniform[];
  viewUniforms: Uniform[];
  drawUniforms: Uniform[];
  userUniforms: Uniform[];
  textureSlots: Uniform[];
};

export type Texture = WebGLTexture & {
  mpWidth: number;
  mpHeight: number;
};

export type FileHandle = {
  id: number;
  basename: string;
  file: File;
  lastReadStart: number;
  lastReadEnd: number;
};

export type WasmEnv = {
  memory: WebAssembly.Memory;
  _consoleLog: (charsPtr: string, len: string, error: any) => void;
  readUserFileRange: (
    userFileId: number,
    bufPtr: number,
    bufLen: number,
    fileOffset: number
  ) => BigInt;
  performanceNow: () => number;
  threadSpawn: (ctxPtr: BigInt) => void;
  _sendEventFromAnyThread: (eventPtr: BigInt) => void;
  readUrlSync: (
    urlPtr: number,
    urlLen: number,
    bufPtrOut: number,
    bufLenOut: number
  ) => 1 | 0;
  randomU64: () => BigInt;
  sendTaskWorkerMessage: (twMessagePtr: string) => void;
};

export type WasmExports = {
  allocWasmVec: (bytes: BigInt) => BigInt;
  allocWasmMessage: (bytes: BigInt) => BigInt;
  deallocWasmMessage: (inBuf: BigInt) => void;
  reallocWasmMessage: (inBuf: BigInt, newBytes: BigInt) => BigInt;
  createWasmApp: () => BigInt;
  processWasmEvents: (appcx: BigInt, msgBytes: BigInt) => BigInt;
  decrementArc: (arcPtr: BigInt) => void;
  callRustInSameThreadSync: (appcx: BigInt, msgBytes: BigInt) => BigInt;
  incrementArc: (arcPtr: BigInt) => void;
  createArcVec: (vecPtr: BigInt, vecLen: BigInt) => BigInt;
  deallocVec: (vecPtr: BigInt, vecLen: BigInt, vecCap: BigInt) => BigInt;
  runFunctionPointer: (ctxPtr: number) => void;
};

// TODO(Paras): This structure is used for both vectors and Arcs for read-only vectors,
// and therefore has optional bufferCap and arcPtr fields. If these structures diverge more,
// we should split this into different types entirely.
export type BufferData = {
  bufferPtr: number;
  bufferLen: number;
  bufferCap: number | null;
  arcPtr: number | null;
};

export type PostMessageTypedArray = {
  bufferData: BufferData;
  byteOffset: number;
  byteLength: number;
};

export type CallJSData = {
  fnName: string;
  params: (string | BufferData)[];
};

export type WrfParam = Uint8Array | string;

export type CallJsCallback = (params: WrfParam[]) => void;

export type CallRust = (
  name: string,
  params?: WrfParam[]
) => Promise<WrfParam[]>;

export type CallRustInSameThreadSync = (
  ...args: Parameters<CallRust>
) => WrfParam[];

export enum WrfParamType {
  String = 0,
  ReadOnlyBuffer = 1,
  Buffer = 2,
}

export type UserWorkerInitReturnValue = {
  wasmModule: WebAssembly.Module;
  memory: WebAssembly.Memory;
  taskWorkerSab: SharedArrayBuffer;
  appPtr: BigInt;
  baseUri: string;
};

export enum WorkerEvent {
  CallRust = "CallRust",
  BindUserWorkerPortOnMainThread = "BindUserWorkerPortOnMainThread",
  DecrementArc = "DecrementArc",
  DeallocVec = "DeallocVec",
  IncrementArc = "IncrementArc",
  DragEnter = "DragEnter",
  DragOver = "DragOver",
  DragLeave = "DragLeave",
  Drop = "Drop",
  WindowMouseUp = "WindowMouseUp",
  CanvasMouseDown = "CanvasMouseDown",
  WindowMouseMove = "WindowMouseMove",
  WindowMouseOut = "WindowMouseOut",
  WindowFocus = "WindowFocus",
  WindowBlur = "WindowBlur",
  ScreenResize = "ScreenResize",
  CanvasWheel = "CanvasWheel",
  ShowIncompatibleBrowserNotification = "ShowIncompatibleBrowserNotification",
  RemoveLoadingIndicators = "RemoveLoadingIndicators",
  SetDocumentTitle = "SetDocumentTitle",
  SetMouseCursor = "SetMouseCursor",
  Fullscreen = "Fullscreen",
  Normalscreen = "Normalscreen",
  TextCopyResponse = "TextCopyResponse",
  EnableGlobalFileDropTarget = "EnableGlobalFileDropTarget",
  CallJs = "CallJs",
  ShowTextIME = "ShowTextIME",
  TextInput = "TextInput",
  TextCopy = "TextCopy",
  KeyDown = "KeyDown",
  KeyUp = "KeyUp",
  Init = "Init",
}

export enum UserWorkerEvent {
  Init = "Init",
  BindUserWorkerPortOnMainThread = "BindUserWorkerPortOnMainThread",
  CallRust = "CallRust",
  SendEventFromAnyThread = "SendEventFromAnyThread",
}

export enum AsyncWorkerEvent {
  SendEventFromAnyThread = "SendEventFromAnyThread",
  Run = "Run",
  ThreadSpawn = "ThreadSpawn",
}

export enum TaskWorkerEvent {
  Init = "Init",
}
