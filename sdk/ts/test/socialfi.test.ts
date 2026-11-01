import test from 'node:test';
import assert from 'node:assert/strict';
import { prepareSocialFiConsent } from '../src/socialfi.js';
import { hashId, consentDigest } from '../src/consent.js';

test('SocialFi consent requires a separately reviewed split hash and binds work identity', () => {
  const args = { workId:'hour:work:one', reviewedSplitVersionHash:new Uint8Array(32).fill(1),
    contributorPublicKey:Uint8Array.from([2,...new Array(32).fill(3)]), nonce:1n, expiry:2000000000n };
  const fields = prepareSocialFiConsent(args);
  assert.deepEqual(fields.releaseId, hashId(args.workId));
  assert.notDeepEqual(consentDigest(fields), consentDigest(prepareSocialFiConsent({...args,workId:'hour:work:two'})));
  assert.throws(() => prepareSocialFiConsent({...args, reviewedSplitVersionHash:new Uint8Array(1)}));
  assert.throws(() => prepareSocialFiConsent({...args, workId:'post:one'}));
  assert.throws(() => prepareSocialFiConsent({...args, workId:'hour:work:'}));
  assert.throws(() => prepareSocialFiConsent({...args, nonce:-1n}));
  args.reviewedSplitVersionHash.fill(0);
  assert.equal(fields.splitVersionHash[0], 1);
});
