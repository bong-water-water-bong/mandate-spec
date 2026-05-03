// mandate-core (TS) — types and verification for the Mandate v0 wire format.
// See ../../docs/specs/2026-05-03-mandate-v0.md for the canonical spec.

export type Money = {
  amount: string;
  currency: string;
};

export type Signature = {
  alg: "eip191" | "ed25519-solana";
  value: string;
};

export type Buyer = {
  wallet: string;
  sig: Signature;
};

export type Intent = {
  brief: string;
  structured?: {
    category?: string;
    must?: Record<string, unknown>;
    prefer?: Record<string, unknown>;
    avoid?: Record<string, unknown>;
  };
};

export type Budget = {
  max_total: Money;
  settlement_rails: string[];
  fees_inclusive?: boolean;
  tip_policy?: "none" | "merchant_default" | "ask";
};

export type VaultGrant = {
  type: string;
  value: string;
};

export type Preferences = {
  vault_pointer?: string;
  vault_grant?: VaultGrant;
};

export type Policies = {
  fulfillment_by?: string;
  return_window_days_min?: number;
  substitutions?: "ok" | "no" | "ask";
  shipping_destinations?: string[];
  data_sharing?: "minimum_necessary" | "full";
};

export type Attestation = {
  type: "amd-sev-snp";
  vcek_chain?: string;
  report: string;
  report_data_binding: "blake3(canonical_body)[0..64]";
};

export type Agent = {
  runtime_id: string;
  attestation: Attestation;
};

export type Approval = {
  human_required_above?: Money;
  approval_sig: Signature | null;
};

export type Lifecycle = {
  created_at: string;
  expires_at: string;
  nonce: string;
  bound_to_merchant: string | null;
  state:
    | "minted"
    | "presented"
    | "quoted"
    | "approved"
    | "fulfilling"
    | "settled"
    | "closed"
    | "expired"
    | "rejected"
    | "refunded";
};

export type Mandate = {
  version: "mandate/v0";
  id: string;
  buyer: Buyer;
  intent: Intent;
  budget: Budget;
  preferences?: Preferences;
  policies?: Policies;
  agent: Agent;
  approval?: Approval;
  lifecycle: Lifecycle;
};

// TODO(v0.0.1): canonicalization (JCS / RFC 8785), id derivation,
// sign/verify (eip191 + ed25519), and SEV-SNP attestation verification.
export const VERSION = "mandate/v0" as const;
