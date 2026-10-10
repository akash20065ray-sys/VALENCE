// Pure TypeScript implementation of MurmurHash3 32-bit & Google Jump Consistent Hash
// Mirrors exact behavior of src/sharding/router.rs

const DEFAULT_SEED = 0x9747b28c;

export function murmur3_32(keyStr: string, seed: number = DEFAULT_SEED): number {
  const encoder = new TextEncoder();
  const key = encoder.encode(keyStr);
  const len = key.length;

  const c1 = 0xcc9e2d51;
  const c2 = 0x1b873593;
  const r1 = 15;
  const r2 = 13;
  const m = 5;
  const n = 0xe6546b64;

  let hash = seed >>> 0;
  const nblocks = Math.floor(len / 4);

  for (let i = 0; i < nblocks; i++) {
    const idx = i * 4;
    let k =
      (key[idx] & 0xff) |
      ((key[idx + 1] & 0xff) << 8) |
      ((key[idx + 2] & 0xff) << 16) |
      ((key[idx + 3] & 0xff) << 24);

    k = Math.imul(k, c1) >>> 0;
    k = ((k << r1) | (k >>> (32 - r1))) >>> 0;
    k = Math.imul(k, c2) >>> 0;

    hash = (hash ^ k) >>> 0;
    hash = ((hash << r2) | (hash >>> (32 - r2))) >>> 0;
    hash = (Math.imul(hash, m) + n) >>> 0;
  }

  let k = 0;
  const rem = len % 4;
  const tailIdx = nblocks * 4;

  if (rem === 3) {
    k ^= (key[tailIdx + 2] & 0xff) << 16;
    k ^= (key[tailIdx + 1] & 0xff) << 8;
    k ^= key[tailIdx] & 0xff;
    k = Math.imul(k, c1) >>> 0;
    k = ((k << r1) | (k >>> (32 - r1))) >>> 0;
    k = Math.imul(k, c2) >>> 0;
    hash = (hash ^ k) >>> 0;
  } else if (rem === 2) {
    k ^= (key[tailIdx + 1] & 0xff) << 8;
    k ^= key[tailIdx] & 0xff;
    k = Math.imul(k, c1) >>> 0;
    k = ((k << r1) | (k >>> (32 - r1))) >>> 0;
    k = Math.imul(k, c2) >>> 0;
    hash = (hash ^ k) >>> 0;
  } else if (rem === 1) {
    k ^= key[tailIdx] & 0xff;
    k = Math.imul(k, c1) >>> 0;
    k = ((k << r1) | (k >>> (32 - r1))) >>> 0;
    k = Math.imul(k, c2) >>> 0;
    hash = (hash ^ k) >>> 0;
  }

  hash = (hash ^ len) >>> 0;
  hash = (hash ^ (hash >>> 16)) >>> 0;
  hash = Math.imul(hash, 0x85ebca6b) >>> 0;
  hash = (hash ^ (hash >>> 13)) >>> 0;
  hash = Math.imul(hash, 0xc2b2ae35) >>> 0;
  hash = (hash ^ (hash >>> 16)) >>> 0;

  return hash >>> 0;
}

export function jumpConsistentHash(keyHash: bigint, numShards: number): number {
  if (numShards <= 0) return 0;
  let b = BigInt(-1);
  let j = BigInt(0);
  const n = BigInt(numShards);
  let k = keyHash;

  const mult = BigInt("2862933555777941757");
  const mask64 = (BigInt(1) << BigInt(64)) - BigInt(1);
  const two31 = BigInt(1) << BigInt(31);

  while (j < n) {
    b = j;
    k = (k * mult + BigInt(1)) & mask64;
    const randVal = Number((k >> BigInt(33)) + BigInt(1));
    j = BigInt(Math.floor((Number(b + BigInt(1)) * Number(two31)) / randVal));
  }

  return Number(b);
}

export function routeKey(key: string, numShards: number): number {
  if (numShards <= 0) return 0;
  const hash = murmur3_32(key);
  return hash % numShards;
}

export function routeKeyJump(key: string, numShards: number): number {
  if (numShards <= 0) return 0;
  const h1 = BigInt(murmur3_32(key, DEFAULT_SEED));
  const h2 = BigInt(murmur3_32(key, (DEFAULT_SEED + 0x5bd1e995) >>> 0));
  const combined = (h1 << BigInt(32)) | h2;
  return jumpConsistentHash(combined, numShards);
}
