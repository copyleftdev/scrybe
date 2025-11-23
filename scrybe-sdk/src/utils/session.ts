/**
 * Session ID generation utilities
 */

/**
 * Generate a unique session identifier (UUID v4)
 * 
 * @returns UUID v4 string
 */
export function generateSessionId(): string {
  // UUID v4 generation
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Store session ID in sessionStorage
 * 
 * @param sessionId - Session identifier to store
 */
export function storeSessionId(sessionId: string): void {
  try {
    sessionStorage.setItem('scrybe_session_id', sessionId);
  } catch {
    // Ignore storage errors
  }
}

/**
 * Retrieve session ID from sessionStorage
 * 
 * @returns Stored session ID or null
 */
export function getStoredSessionId(): string | null {
  try {
    return sessionStorage.getItem('scrybe_session_id');
  } catch {
    return null;
  }
}
