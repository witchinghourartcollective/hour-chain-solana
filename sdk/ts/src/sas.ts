/**
 * Draft Solana Attestation Service (SAS) schema definitions for hOUR creator-rights records.
 * STATUS: specification only (docs/spec.md §5). No SAS credential or schema has been created
 * on any cluster yet; issuing/verifying via sas-lib is Milestone 2.
 */
export interface DraftSasSchema {
  name: string;
  version: number;
  description: string;
  fields: ReadonlyArray<{ name: string; type: "bytes32" | "u64" | "i64" | "u16" | "string" | "bytes33" }>;
}

export const DRAFT_SAS_SCHEMAS: ReadonlyArray<DraftSasSchema> = [
  {
    name: "hour.contributor-credit",
    version: 1,
    description: "A contributor is credited with a role on a release.",
    fields: [
      { name: "release_id", type: "bytes32" },
      { name: "contributor_id", type: "bytes32" },
      { name: "role", type: "string" },
      { name: "hour_event_hash", type: "bytes32" },
    ],
  },
  {
    name: "hour.split-version",
    version: 1,
    description: "A versioned split document for a release and rights category.",
    fields: [
      { name: "release_id", type: "bytes32" },
      { name: "split_version_hash", type: "bytes32" },
      { name: "rights_category", type: "string" },
      { name: "version", type: "u64" },
      { name: "hour_event_hash", type: "bytes32" },
    ],
  },
  {
    name: "hour.consent-receipt",
    version: 1,
    description: "Links an hour-consent record PDA to an hOUR (optionally ML-DSA-65-signed) event.",
    fields: [
      { name: "release_id", type: "bytes32" },
      { name: "split_version_hash", type: "bytes32" },
      { name: "contributor_id", type: "bytes32" },
      { name: "consent_digest", type: "bytes32" },
      { name: "hour_event_hash", type: "bytes32" },
    ],
  },
  {
    name: "hour.settlement-receipt",
    version: 1,
    description: "Links a payment receipt on the settlement network (Base, USDC) to a consented split version.",
    fields: [
      { name: "release_id", type: "bytes32" },
      { name: "split_version_hash", type: "bytes32" },
      { name: "settlement_network", type: "string" },
      { name: "settlement_tx", type: "string" },
      { name: "instruction_hash", type: "bytes32" },
    ],
  },
];
