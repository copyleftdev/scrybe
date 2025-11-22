# RFC-0002: JavaScript SDK (Browser Collection Agent)

- **Status**: Draft
- **Version**: 0.2.0
- **Author**: Zuub Engineering
- **Created**: 2025-01-22
- **Updated**: 2025-01-22
- **Depends On**: RFC-0001 v0.2.0
- **Review**: Addressed security vulnerabilities and anti-spoofing measures

## Summary

The Scrybe JavaScript SDK is a lightweight, non-blocking browser agent that collects multi-layer signals (network, browser, behavioral) and transmits them to the Rust ingestion gateway. It operates passively, respects privacy, and adds < 20ms overhead to page load.

## Motivation

Bot detection requires comprehensive client-side signals that can't be collected server-side:
- Canvas/WebGL/Audio fingerprints
- User behavioral patterns (mouse, scroll, timing)
- Browser API availability and quirks
- Client-side entropy measurements

The SDK must be:
1. **Lightweight**: < 30KB gzipped
2. **Non-blocking**: Async collection, beacon transport
3. **Privacy-aware**: No PII, no input tracking
4. **Resilient**: Works even if ingestion fails

## Design Goals

1. **Comprehensive Signal Collection**: Capture all relevant fingerprint components
2. **Performance**: < 20ms initialization, non-blocking collection
3. **Privacy**: No PII, salted hashes, opt-out support
4. **Reliability**: Graceful degradation, retry logic
5. **Developer Experience**: Simple integration, TypeScript types

## Signal Categories

### 1. Network Signals (Client-Side Observable)

```typescript
interface NetworkSignals {
  // Connection info
  effectiveType: string;         // '4g', 'wifi', etc.
  downlink: number;              // Mbps
  rtt: number;                   // Round-trip time
  
  // Protocol hints
  httpVersion: string;           // From Performance API
  
  // Timing
  navigationTiming: NavigationTiming;
  resourceTiming: ResourceTiming[];
}
```

### 2. Browser Signals (Environment Fingerprint)

```typescript
interface BrowserSignals {
  // Canvas fingerprint
  canvas: {
    hash: string;                // SHA-256 of canvas rendering
    supported: boolean;
  };
  
  // WebGL fingerprint
  webgl: {
    hash: string;                // SHA-256 of WebGL params
    vendor: string;
    renderer: string;
    supportedExtensions: string[];
  };
  
  // Audio fingerprint
  audio: {
    hash: string;                // SHA-256 of audio context
    supported: boolean;
  };
  
  // Fonts
  fonts: {
    available: string[];         // Detected fonts
    hash: string;                // Hash of font list
  };
  
  // Screen & viewport
  screen: {
    width: number;
    height: number;
    colorDepth: number;
    pixelRatio: number;
    orientation: string;
  };
  
  // Browser environment
  navigator: {
    userAgent: string;
    language: string;
    languages: string[];
    platform: string;
    hardwareConcurrency: number;
    deviceMemory: number;
    maxTouchPoints: number;
  };
  
  // Browser quirks
  quirks: {
    webdriver: boolean;          // navigator.webdriver
    automation: boolean;         // Various detection methods
    phantom: boolean;            // PhantomJS detection
    selenium: boolean;           // Selenium detection
  };
  
  // Storage
  storage: {
    localStorage: boolean;
    sessionStorage: boolean;
    indexedDB: boolean;
    cookies: boolean;
  };
  
  // Plugins
  plugins: {
    count: number;
    list: string[];              // Plugin names only
  };
}
```

### 3. Behavioral Signals (Human Interaction Patterns)

```typescript
interface BehavioralSignals {
  // Mouse movement (bounded to prevent DoS)
  mouse: {
    events: MouseEvent[];        // Max 100 events (enforced)
    entropy: number;             // Shannon entropy of movement
    velocity: number[];          // Max 50 samples
    acceleration: number[];      // Max 50 samples
    jerk: number[];              // Rate of acceleration change
  };
  
  // Scroll behavior (bounded)
  scroll: {
    events: ScrollEvent[];       // Max 50 events (enforced)
    velocity: number[];          // Max 30 samples
    smoothness: number;          // Measure of scroll smoothness
  };
  
  // Click patterns (bounded)
  clicks: {
    events: ClickEvent[];        // Max 20 clicks (enforced)
    density: number;             // Clicks per area
    timing: number[];            // Inter-click intervals
  };
  
  // Keyboard (NO input values, only patterns)
  keyboard: {
    eventCount: number;
    avgTimeBetweenKeys: number;
    hasActivity: boolean;
  };
  
  // Timing patterns
  timing: {
    timeOnPage: number;
    idleTime: number;
    activeTime: number;
    focusChanges: number;
    visibilityChanges: number;
  };
  
  // Page interaction
  interaction: {
    scrollDepth: number;         // Max scroll percentage
    maxScrollVelocity: number;
    elementsClicked: number;
    formsInteracted: number;     // Count only, no values
  };
}
```

## SDK Architecture

```
┌─────────────────────────────────────────┐
│           Scrybe SDK (Browser)          │
├─────────────────────────────────────────┤
│  Initialization                         │
│  ├─ Load collectors                     │
│  ├─ Generate session ID                 │
│  └─ Check opt-out                       │
├─────────────────────────────────────────┤
│  Signal Collectors (Async)              │
│  ├─ Network Collector                   │
│  ├─ Browser Collector                   │
│  └─ Behavioral Collector                │
├─────────────────────────────────────────┤
│  Fingerprint Generator                  │
│  ├─ Combine all signals                 │
│  ├─ Compute composite hash              │
│  └─ Calculate confidence                │
├─────────────────────────────────────────┤
│  Transport Layer                        │
│  ├─ Beacon API (primary)                │
│  ├─ Fetch API (fallback)                │
│  └─ Retry queue                         │
└─────────────────────────────────────────┘
```

## Implementation

### SDK Initialization

```typescript
// Auto-initialize on script load
(function() {
  // Check opt-out
  if (window.scrybeOptOut || navigator.doNotTrack === '1') {
    return;
  }
  
  // Initialize SDK
  const scrybe = new ScrybeSDK({
    endpoint: 'https://scrybe.example.com/api/v1/ingest',
    sessionId: generateSessionId(),
    debug: false,
  });
  
  // Start collection
  scrybe.collect();
})();
```

### Session ID Generation

```typescript
function generateSessionId(): string {
  // Check existing session
  const existing = localStorage.getItem('scrybe_session_id');
  if (existing && isValidSession(existing)) {
    return existing;
  }
  
  // Generate new UUID
  const sessionId = crypto.randomUUID();
  localStorage.setItem('scrybe_session_id', sessionId);
  localStorage.setItem('scrybe_session_started', Date.now().toString());
  
  return sessionId;
}
```

### Canvas Fingerprinting (Anti-Spoofing)

```typescript
function collectCanvasFingerprint(): { hash: string; supported: boolean; tests: string[] } {
  try {
    const tests: string[] = [];
    
    // Test 1: Text rendering with emoji
    const canvas1 = document.createElement('canvas');
    canvas1.width = 200;
    canvas1.height = 50;
    const ctx1 = canvas1.getContext('2d')!;
    ctx1.textBaseline = 'top';
    ctx1.font = '14px "Arial"';
    ctx1.fillStyle = '#f60';
    ctx1.fillRect(125, 1, 62, 20);
    ctx1.fillStyle = '#069';
    ctx1.fillText('Scrybe 🔍', 2, 15);
    tests.push(canvas1.toDataURL());
    
    // Test 2: Geometric shapes with gradients
    const canvas2 = document.createElement('canvas');
    canvas2.width = 100;
    canvas2.height = 100;
    const ctx2 = canvas2.getContext('2d')!;
    const gradient = ctx2.createLinearGradient(0, 0, 100, 100);
    gradient.addColorStop(0, 'red');
    gradient.addColorStop(1, 'blue');
    ctx2.fillStyle = gradient;
    ctx2.fillRect(0, 0, 100, 100);
    ctx2.beginPath();
    ctx2.arc(50, 50, 30, 0, Math.PI * 2);
    ctx2.fill();
    tests.push(canvas2.toDataURL());
    
    // Test 3: Bezier curves (hard to spoof consistently)
    const canvas3 = document.createElement('canvas');
    canvas3.width = 100;
    canvas3.height = 100;
    const ctx3 = canvas3.getContext('2d')!;
    ctx3.beginPath();
    ctx3.moveTo(20, 20);
    ctx3.bezierCurveTo(20, 100, 200, 100, 200, 20);
    ctx3.stroke();
    tests.push(canvas3.toDataURL());
    
    // Combine all tests
    const combined = tests.join('|');
    const hash = await sha256(combined);
    
    return { hash, supported: true, tests: tests.map((_, i) => `test${i+1}`) };
  } catch (e) {
    return { hash: '', supported: false, tests: [] };
  }
}
```

### WebGL Fingerprinting

```typescript
function collectWebGLFingerprint(): WebGLFingerprint {
  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    
    if (!gl) {
      return { hash: '', vendor: '', renderer: '', supportedExtensions: [] };
    }
    
    // Collect WebGL parameters
    const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
    const vendor = debugInfo ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) : '';
    const renderer = debugInfo ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) : '';
    
    // Get supported extensions
    const extensions = gl.getSupportedExtensions() || [];
    
    // Combine into fingerprint
    const fingerprint = JSON.stringify({
      vendor,
      renderer,
      extensions: extensions.sort(),
    });
    
    const hash = await sha256(fingerprint);
    
    return {
      hash,
      vendor,
      renderer,
      supportedExtensions: extensions,
    };
  } catch (e) {
    return { hash: '', vendor: '', renderer: '', supportedExtensions: [] };
  }
}
```

### Audio Fingerprinting

```typescript
function collectAudioFingerprint(): { hash: string; supported: boolean } {
  try {
    const context = new (window.AudioContext || window.webkitAudioContext)();
    
    // Create oscillator
    const oscillator = context.createOscillator();
    oscillator.type = 'triangle';
    oscillator.frequency.value = 10000;
    
    // Create compressor
    const compressor = context.createDynamicsCompressor();
    compressor.threshold.value = -50;
    compressor.knee.value = 40;
    compressor.ratio.value = 12;
    compressor.attack.value = 0;
    compressor.release.value = 0.25;
    
    // Connect nodes
    oscillator.connect(compressor);
    compressor.connect(context.destination);
    
    // Start and get data
    oscillator.start(0);
    const analyser = context.createAnalyser();
    compressor.connect(analyser);
    
    const buffer = new Float32Array(analyser.fftSize);
    analyser.getFloatTimeDomainData(buffer);
    
    oscillator.stop();
    context.close();
    
    // Hash the audio signature
    const hash = await sha256(buffer.join(','));
    
    return { hash, supported: true };
  } catch (e) {
    return { hash: '', supported: false };
  }
}
```

### Behavioral Collection (Mouse) - Shannon Entropy

```typescript
class BehavioralCollector {
  private mouseEvents: MouseEvent[] = [];
  private readonly MAX_MOUSE_EVENTS = 100;
  private readonly MAX_VELOCITY_SAMPLES = 50;
  
  startCollection() {
    document.addEventListener('mousemove', this.handleMouseMove.bind(this), { passive: true });
    document.addEventListener('click', this.handleClick.bind(this), { passive: true });
  }
  
  private handleMouseMove(event: MouseEvent) {
    // Enforce bounded collection
    if (this.mouseEvents.length >= this.MAX_MOUSE_EVENTS) {
      // Shift out oldest event
      this.mouseEvents.shift();
    }
    
    this.mouseEvents.push({
      timestamp: Date.now(),
      x: event.clientX,
      y: event.clientY,
      // NO element info, NO target info (privacy)
    });
  }
  
  private calculateEntropy(): number {
    // Shannon entropy of velocity vectors
    if (this.mouseEvents.length < 2) return 0;
    
    // Calculate velocities
    const velocities: number[] = [];
    for (let i = 1; i < this.mouseEvents.length; i++) {
      const dx = this.mouseEvents[i].x - this.mouseEvents[i-1].x;
      const dy = this.mouseEvents[i].y - this.mouseEvents[i-1].y;
      const dt = this.mouseEvents[i].timestamp - this.mouseEvents[i-1].timestamp;
      if (dt > 0) {
        velocities.push(Math.sqrt(dx*dx + dy*dy) / dt);
      }
    }
    
    // Quantize velocities into bins
    const bins = 20;
    const maxVel = Math.max(...velocities);
    const histogram = new Array(bins).fill(0);
    
    for (const vel of velocities) {
      const binIndex = Math.min(bins - 1, Math.floor((vel / maxVel) * bins));
      histogram[binIndex]++;
    }
    
    // Calculate Shannon entropy
    let entropy = 0;
    const total = velocities.length;
    for (const count of histogram) {
      if (count > 0) {
        const p = count / total;
        entropy -= p * Math.log2(p);
      }
    }
    
    // Normalize to 0-1
    return entropy / Math.log2(bins);
  }
}
```

### Transport Layer (Beacon API)

```typescript
class Transport {
  async send(sessionData: Session): Promise<void> {
    const payload = JSON.stringify(sessionData);
    
    // Try Beacon API first (non-blocking)
    if (navigator.sendBeacon) {
      const sent = navigator.sendBeacon(this.endpoint, payload);
      if (sent) {
        return;
      }
    }
    
    // Fallback to Fetch with keepalive
    try {
      await fetch(this.endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: payload,
        keepalive: true,
      });
    } catch (e) {
      // Queue for retry
      this.queueForRetry(sessionData);
    }
  }
  
  private queueForRetry(session: Session) {
    const queue = JSON.parse(localStorage.getItem('scrybe_retry_queue') || '[]');
    queue.push(session);
    
    // Keep max 10 in queue
    if (queue.length > 10) {
      queue.shift();
    }
    
    localStorage.setItem('scrybe_retry_queue', JSON.stringify(queue));
  }
}
```

## Privacy Safeguards

### 1. No PII Collection
```typescript
// ❌ NEVER collect:
// - Input field values
// - Form data
// - URLs with query params
// - Cookies
// - localStorage keys/values (except our own)

// ✅ DO collect:
// - Anonymized interaction patterns
// - Browser capabilities
// - Timing data
// - Canvas/WebGL hashes
```

### 2. Opt-Out Mechanisms
```typescript
// Check multiple opt-out signals
function shouldOptOut(): boolean {
  return (
    window.scrybeOptOut === true ||
    navigator.doNotTrack === '1' ||
    document.querySelector('meta[name="scrybe-opt-out"]') !== null
  );
}
```

### 3. Data Minimization
```typescript
// Limit event collection
const MAX_MOUSE_EVENTS = 100;
const MAX_SCROLL_EVENTS = 50;
const MAX_CLICK_EVENTS = 20;

// Truncate after limits
if (mouseEvents.length > MAX_MOUSE_EVENTS) {
  mouseEvents = mouseEvents.slice(0, MAX_MOUSE_EVENTS);
}
```

## Performance Optimization

### 1. Async Collection
```typescript
async function collect(): Promise<Session> {
  // Collect in parallel
  const [network, browser, behavioral] = await Promise.all([
    collectNetworkSignals(),
    collectBrowserSignals(),
    collectBehavioralSignals(),
  ]);
  
  return { network, browser, behavioral };
}
```

### 2. Lazy Loading
```typescript
// Only collect behavioral signals after user interaction
let behavioralCollectorStarted = false;

document.addEventListener('mousemove', () => {
  if (!behavioralCollectorStarted) {
    startBehavioralCollection();
    behavioralCollectorStarted = true;
  }
}, { once: true, passive: true });
```

### 3. Debouncing
```typescript
// Debounce transmission
const debouncedSend = debounce(transport.send, 5000);

// Send after 5s of inactivity or on page unload
window.addEventListener('beforeunload', () => {
  transport.send(currentSession);
});
```

## Bundle Size

Target: < 30KB gzipped

- Core SDK: 15KB
- Network collector: 3KB
- Browser collector: 8KB
- Behavioral collector: 4KB

Total: ~30KB gzipped

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Graceful degradation for older browsers.

### Anti-Replay Protection

```typescript
interface SessionPayload {
  sessionId: string;
  timestamp: number;
  nonce: string;              // Server must validate uniqueness
  signals: CollectedSignals;
  signature: string;           // HMAC of above fields
}

async function createSignedPayload(signals: CollectedSignals): Promise<SessionPayload> {
  const sessionId = getSessionId();
  const timestamp = Date.now();
  const nonce = crypto.randomUUID();
  
  // Create payload without signature
  const payload = {
    sessionId,
    timestamp,
    nonce,
    signals,
  };
  
  // Sign payload (requires server-provided key on init)
  const signature = await signPayload(payload, clientKey);
  
  return { ...payload, signature };
}

// Server validates:
// 1. Signature is valid
// 2. Timestamp is recent (< 5 minutes old)
// 3. Nonce has never been seen before (Redis check)
```

## Cookie Consent Integration

```typescript
// Check for consent before setting cookies
function hasConsent(): boolean {
  // Check common consent management platforms
  
  // OneTrust
  if ((window as any).OneTrust) {
    const activeGroups = (window as any).OnetrustActiveGroups;
    return activeGroups?.includes('C0002'); // Performance cookies
  }
  
  // Cookiebot
  if ((window as any).Cookiebot) {
    return (window as any).Cookiebot.consent?.statistics === true;
  }
  
  // CookieYes
  if ((window as any).CookieYes) {
    return (window as any).CookieYes.getConsent('performance');
  }
  
  // Default: Check for generic consent cookie
  return document.cookie.includes('scrybe_consent=true');
}

function initialize() {
  // Only set cookies if consent granted
  if (hasConsent()) {
    setSessionCookie(sessionId);
  } else {
    // Use sessionStorage instead (not persistent)
    sessionStorage.setItem('scrybe_session', sessionId);
  }
}
```

## Integration Example

```html
<!-- Simple integration with SRI -->
<script src="https://cdn.scrybe.com/sdk/v1/scrybe.min.js"
        integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC"
        crossorigin="anonymous"></script>
<script>
  Scrybe.init({
    endpoint: 'https://api.scrybe.com/v1/ingest',
    debug: false,
  });
</script>
```

```typescript
// NPM integration
import Scrybe from '@scrybe/sdk';

Scrybe.init({
  endpoint: process.env.SCRYBE_ENDPOINT,
  debug: process.env.NODE_ENV === 'development',
});
```

## Testing

### Unit Tests
- Each collector independently
- Fingerprint generation
- Transport layer with mocks

### Integration Tests
- Full collection cycle
- Beacon API fallback
- Retry logic

### Browser Tests (Playwright)
- Cross-browser fingerprint consistency
- Performance benchmarks
- Privacy compliance

## Success Criteria

1. ✅ < 30KB bundle size (gzipped)
2. ✅ < 20ms initialization time
3. ✅ Non-blocking collection (no FPS drops)
4. ✅ No PII collected
5. ✅ Graceful degradation
6. ✅ Works in all major browsers
7. ✅ Opt-out respected
8. ✅ Bounded collections (no DoS via memory)
9. ✅ Anti-replay protection (nonce + signature)
10. ✅ Multiple canvas tests (anti-spoofing)
11. ✅ Shannon entropy calculated correctly
12. ✅ Consent management integrated

## References

- RFC-0001: Core Architecture
- RFC-0003: Rust Ingestion Gateway
- FingerprintJS: https://github.com/fingerprintjs/fingerprintjs
- Canvas Fingerprinting: https://browserleaks.com/canvas
- WebGL Fingerprinting: https://browserleaks.com/webgl
