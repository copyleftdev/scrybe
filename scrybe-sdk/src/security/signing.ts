/**
 * Payload signing for HMAC authentication
 */

/**
 * Generate a random nonce (UUID v4)
 * 
 * @returns UUID v4 string
 */
export function generateNonce(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Sign payload using HMAC-SHA256
 * 
 * @param body - Request body
 * @param timestamp - Unix timestamp in milliseconds
 * @param nonce - Random nonce
 * @param apiKey - API key for signing
 * @returns HMAC signature (hex-encoded)
 */
export async function signPayload(
  body: string,
  timestamp: number,
  nonce: string,
  apiKey: string
): Promise<string> {
  // Construct message: timestamp:nonce:body
  const message = `${timestamp}:${nonce}:${body}`;

  // Convert API key to bytes
  const keyData = new TextEncoder().encode(apiKey);

  // Import key for HMAC
  const key = await crypto.subtle.importKey('raw', keyData, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);

  // Sign message
  const messageData = new TextEncoder().encode(message);
  const signature = await crypto.subtle.sign('HMAC', key, messageData);

  // Convert to hex
  const hashArray = Array.from(new Uint8Array(signature));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}
