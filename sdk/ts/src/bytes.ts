export function concatBytes(...parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  return out;
}

export function u16le(v: number): Uint8Array {
  if (!Number.isInteger(v) || v < 0 || v > 0xffff) throw new RangeError(`u16 out of range: ${v}`);
  return new Uint8Array([v & 0xff, v >> 8]);
}

export function u64le(v: bigint): Uint8Array {
  if (v < 0n || v > 0xffff_ffff_ffff_ffffn) throw new RangeError("u64 out of range");
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigUint64(0, v, true);
  return b;
}

export function i64le(v: bigint): Uint8Array {
  if (v < -(2n ** 63n) || v >= 2n ** 63n) throw new RangeError("i64 out of range");
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigInt64(0, v, true);
  return b;
}

export function assertLen(name: string, b: Uint8Array, len: number): void {
  if (!(b instanceof Uint8Array) || b.length !== len) {
    throw new TypeError(`${name} must be ${len} bytes`);
  }
}

export const toHex = (b: Uint8Array): string => Buffer.from(b).toString("hex");
export const fromHex = (h: string): Uint8Array => new Uint8Array(Buffer.from(h, "hex"));
