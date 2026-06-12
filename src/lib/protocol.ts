type Sid = number; // u32
type Uid = number; // u32

/** Position and size of a window, see the Rust version. */
export type WsWinsize = {
  x: number;
  y: number;
  rows: number;
  cols: number;
};

/** Information about a user, see the Rust version */
export type WsUser = {
  name: string;
  cursor: [number, number] | null;
  focus: number | null;
  canWrite: boolean;
};

/** A collaborative board item (image / live stream frame) — maw share extension. */
export type BoardItem = {
  id: string;
  kind: string; // "image" | "stream"
  x: number;
  y: number;
  w: number;
  dataUrl: string;
};

/** Server message type, see the Rust version. */
export type WsServer = {
  hello?: [Uid, string];
  invalidAuth?: [];
  users?: [Uid, WsUser][];
  userDiff?: [Uid, WsUser | null];
  shells?: [Sid, WsWinsize][];
  chunks?: [Sid, number, Uint8Array[]];
  hear?: [Uid, string, string];
  shellLatency?: number | bigint;
  pong?: number | bigint;
  // ── maw share workboard extensions ──
  voiceData?: [Uid, Uint8Array];
  board?: BoardItem[];
  boardPut?: BoardItem;
  boardMove?: [string, number, number];
  boardDelete?: string;
  error?: string;
};

/** Client message type, see the Rust version. */
export type WsClient = {
  authenticate?: [Uint8Array, Uint8Array | null];
  setName?: string;
  setCursor?: [number, number] | null;
  setFocus?: number | null;
  create?: [number, number];
  close?: Sid;
  move?: [Sid, WsWinsize | null];
  data?: [Sid, Uint8Array, bigint];
  subscribe?: [Sid, number];
  chat?: string;
  ping?: bigint;
  // ── maw share workboard extensions ──
  voice?: Uint8Array;
  boardPut?: BoardItem;
  boardMove?: [string, number, number];
  boardDelete?: string;
};
