/**
 * Hashing utilities for fingerprinting
 */

/**
 * Generate SHA-256 hash of a string (using SubtleCrypto API)
 * 
 * @param data - Data to hash
 * @returns Hex-encoded hash
 */
export async function sha256(data: string): Promise<string> {
  const encoder = new TextEncoder();
  const dataBuffer = encoder.encode(data);
  const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Simple hash function (non-cryptographic, fast)
 * Used for quick fingerprinting when crypto API is not needed
 * 
 * @param str - String to hash
 * @returns Numeric hash
 */
export function simpleHash(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  return Math.abs(hash);
}
