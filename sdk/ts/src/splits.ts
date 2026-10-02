/**
 * USDC split payout REFERENCE math (pure functions, no chain calls, moves no funds).
 * Settlement network for hOUR Chain payments is Base; the resulting receipt is meant to be
 * attested back on Solana (hour.settlement-receipt). Executing payments is out of scope here.
 */
import { sha256 } from "@noble/hashes/sha2.js";
import { toHex } from "./bytes.js";

export const BPS_TOTAL = 10_000;
export const USDC_DECIMALS = 6;

export interface SplitShare {
  payee: string;
  bps: number;
}

export interface PayoutLine {
  payee: string;
  bps: number;
  amount: bigint;
}

export function validateSplit(shares: SplitShare[]): void {
  if (shares.length === 0) throw new Error("split must have at least one share");
  const seen = new Set<string>();
  let total = 0;
  for (const s of shares) {
    if (!Number.isInteger(s.bps) || s.bps <= 0) throw new Error(`invalid bps for ${s.payee}`);
    if (seen.has(s.payee)) throw new Error(`duplicate payee ${s.payee}`);
    seen.add(s.payee);
    total += s.bps;
  }
  if (total !== BPS_TOTAL) throw new Error(`split must total ${BPS_TOTAL} bps, got ${total}`);
}

/**
 * Deterministic largest-remainder allocation of `totalAtomic` (e.g. USDC base units).
 * Sum of outputs always equals the input; ties broken by input order.
 */
export function allocate(totalAtomic: bigint, shares: SplitShare[]): PayoutLine[] {
  validateSplit(shares);
  if (totalAtomic < 0n) throw new Error("amount must be non-negative");
  const lines = shares.map((s, i) => {
    const exact = totalAtomic * BigInt(s.bps);
    return { i, payee: s.payee, bps: s.bps, amount: exact / BigInt(BPS_TOTAL), rem: exact % BigInt(BPS_TOTAL) };
  });
  let leftover = totalAtomic - lines.reduce((n, l) => n + l.amount, 0n);
  const order = [...lines].sort((a, b) => (b.rem === a.rem ? a.i - b.i : b.rem > a.rem ? 1 : -1));
  for (const l of order) {
    if (leftover === 0n) break;
    l.amount += 1n;
    leftover -= 1n;
  }
  return lines.map(({ payee, bps, amount }) => ({ payee, bps, amount }));
}

export interface SettlementInstruction {
  kind: "hour.settlement-instruction";
  version: 1;
  network: "base";
  asset: "USDC";
  releaseId: string;
  splitVersionHash: string;
  totalAtomic: string;
  lines: Array<{ payee: string; bps: number; amount: string }>;
  requiresApproval: true;
  simulateFirst: true;
}

export function buildSettlementInstruction(args: {
  releaseId: Uint8Array;
  splitVersionHash: Uint8Array;
  totalAtomic: bigint;
  shares: SplitShare[];
}): { instruction: SettlementInstruction; instructionHash: string } {
  const lines = allocate(args.totalAtomic, args.shares);
  const instruction: SettlementInstruction = {
    kind: "hour.settlement-instruction",
    version: 1,
    network: "base",
    asset: "USDC",
    releaseId: toHex(args.releaseId),
    splitVersionHash: toHex(args.splitVersionHash),
    totalAtomic: args.totalAtomic.toString(),
    lines: lines.map((l) => ({ payee: l.payee, bps: l.bps, amount: l.amount.toString() })),
    requiresApproval: true,
    simulateFirst: true,
  };
  // Keys are emitted in a fixed order above, so JSON.stringify is stable for this shape.
  const instructionHash = toHex(sha256(new TextEncoder().encode(JSON.stringify(instruction))));
  return { instruction, instructionHash };
}
