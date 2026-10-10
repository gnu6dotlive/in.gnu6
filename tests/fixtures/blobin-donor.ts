/**
 * Revision-bound donor fixture extracted from the blob.in snapshot
 * /mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in.
 *
 * Snapshot manifest:
 *   95 files
 *   sha256(manifest) = 5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d
 *
 * This is evidence input, not engine authority.
 */
export const BLOBIN_DONOR_FIXTURE = {
  snapshotLabel: "appstack-b3-baseline-4603aed",
  // biome-ignore lint/security/noSecrets: revision-bound SHA-256 digest, not a credential.
  snapshotManifestSha256: "5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d",
  packageVersion: "0.14.0",
  spring: {
    token: "SETTLE",
    stiffness: 170,
    damping: 22,
    mass: 1,
    startPosition: 0,
    startVelocity: 0,
    target: 1,
    dtSeconds: 1 / 240,
    expectedVelocityAfterOneSubstep: 17 / 24,
    expectedPositionAfterOneSubstep: 17 / 5760,
  },
  morph: {
    subjectId: "blobin-donor-subject",
    source: { kind: "rect" as const, width: 2, height: 1, radius: 0.2 },
    target: { kind: "rect" as const, width: 1, height: 1.4, radius: 0.32 },
    sampleCount: 48,
  },
} as const;
