# SocialFi consent boundary

SocialFi is a first-class creator-community surface in hOUR Chain. Its profiles,
posts, follows, creative-time records and support intents are application records,
not rights approvals, token rewards or settled payments.

The SDK exports `prepareSocialFiConsent({workId, reviewedSplitVersionHash,
contributorPublicKey, nonce, expiry})`. It maps a registered work ID and independently
reviewed split document to the existing raw-digest consent fields. The split document
must commit to the exact work, terms and relevant contribution/event references.
The helper cannot infer rights shares from minutes, posts or follower counts.

The M1 contributor identifier is derived from one P-256 key; it is not a complete
identity, recovery or credential-rotation registry. Real browser WebAuthn, deployed
program/upgrade custody, finality and Cosmos acceptance checks remain launch gates.
No transaction is submitted by this helper. A local app owner clicking Mark reviewed
is not a signed collaborator agreement. See LAUNCH-GATES.md and hOUR-Chain/docs/SOCIALFI.md.
