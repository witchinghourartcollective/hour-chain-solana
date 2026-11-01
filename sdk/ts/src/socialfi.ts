import { hashId, contributorKeyId, type ConsentFields } from './consent.js';
import { assertLen } from './bytes.js';

/**
 * Prepare a raw-digest consent request for an independently reviewed split document.
 * Social posts, likes, follow counts and recorded minutes cannot set rights or payouts.
 * The supplied split hash must commit to the work, terms and contribution/event refs.
 * This helper neither verifies browser WebAuthn nor submits an instruction.
 */
export function prepareSocialFiConsent(args: {
  workId: string; reviewedSplitVersionHash: Uint8Array;
  contributorPublicKey: Uint8Array; nonce: bigint; expiry: bigint;
}): ConsentFields {
  if (typeof args.workId !== 'string' || !args.workId.startsWith('hour:work:') || args.workId.length > 300) {
    throw new TypeError('A registered hOUR work identifier is required');
  }
  assertLen('reviewedSplitVersionHash', args.reviewedSplitVersionHash, 32);
  if (args.nonce < 0n || args.nonce >= 2n ** 64n || args.expiry <= 0n || args.expiry >= 2n ** 63n) {
    throw new TypeError('Invalid consent nonce or expiry');
  }
  return { releaseId: hashId(args.workId), splitVersionHash: new Uint8Array(args.reviewedSplitVersionHash),
    contributorId: contributorKeyId(args.contributorPublicKey), nonce: args.nonce, expiry: args.expiry };
}
