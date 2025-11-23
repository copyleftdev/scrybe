---
trigger: always_on
---

# JavaScript/TypeScript SDK Rules

**Activation**: Glob pattern `*.ts`, `*.js`, `*.tsx`, `*.jsx`

## TypeScript Standards

### Strict Mode Always
```typescript
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "noImplicitAny": true,
    "noImplicitThis": true,
    "alwaysStrict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

### Type Safety
```typescript
// ✅ GOOD - Explicit types
export interface BrowserTelemetry {
    userAgent: string;
    screenWidth: number;
    screenHeight: number;
    colorDepth: number;
    timezone: string;
    language: string;
    plugins: readonly string[];
}

export function collectTelemetry(): BrowserTelemetry {
    return {
        userAgent: navigator.userAgent,
        screenWidth: window.screen.width,
        screenHeight: window.screen.height,
        colorDepth: window.screen.colorDepth,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        language: navigator.language,
        plugins: Array.from(navigator.plugins).map(p => p.name),
    };
}

// ❌ BAD - Using 'any'
export function collectTelemetry(): any {
    return {
        userAgent: navigator.userAgent,
        // ...
    };
}
```

## Bounded Collections (Security)

### Prevent Unbounded Growth
```typescript
/**
 * Bounded array with maximum capacity to prevent memory exhaustion
 */
export class BoundedArray<T> {
    private readonly items: T[] = [];
    
    constructor(private readonly maxSize: number) {
        if (maxSize <= 0) {
            throw new Error('maxSize must be positive');
        }
    }
    
    push(item: T): boolean {
        if (this.items.length >= this.maxSize) {
            return false;
        }
        this.items.push(item);
        return true;
    }
    
    get length(): number {
        return this.items.length;
    }
    
    toArray(): readonly T[] {
        return [...this.items];
    }
}

// Usage in SDK
const MAX_EVENTS = 1000;
const events = new BoundedArray<BehaviorEvent>(MAX_EVENTS);
```

## Event Collection Patterns

### Mouse Event Sampling
```typescript
interface MouseEvent {
    timestamp: number;
    x: number;
    y: number;
    type: 'move' | 'click' | 'scroll';
}

export class MouseTracker {
    private readonly events: BoundedArray<MouseEvent>;
    private lastSampleTime = 0;
    private readonly sampleIntervalMs = 100; // Sample every 100ms
    
    constructor(maxEvents: number = 1000) {
        this.events = new BoundedArray(maxEvents);
        this.attachListeners();
    }
    
    private attachListeners(): void {
        // Throttled mouse move
        document.addEventListener('mousemove', this.handleMouseMove.bind(this), { passive: true });
        
        // Capture all clicks
        document.addEventListener('click', this.handleClick.bind(this), { passive: true });
    }
    
    private handleMouseMove(event: globalThis.MouseEvent): void {
        const now = Date.now();
        
        // Throttle to avoid overwhelming the queue
        if (now - this.lastSampleTime < this.sampleIntervalMs) {
            return;
        }
        
        this.lastSampleTime = now;
        this.events.push({
            timestamp: now,
            x: event.clientX,
            y: event.clientY,
            type: 'move',
        });
    }
    
    private handleClick(event: globalThis.MouseEvent): void {
        this.events.push({
            timestamp: Date.now(),
            x: event.clientX,
            y: event.clientY,
            type: 'click',
        });
    }
    
    getEvents(): readonly MouseEvent[] {
        return this.events.toArray();
    }
}
```

## Fingerprint Generation

### Canvas Fingerprinting
```typescript
/**
 * Generate canvas fingerprint with anti-spoofing measures
 * 
 * @returns Base64-encoded SHA-256 hash of canvas data
 */
export async function generateCanvasFingerprint(): Promise<string> {
    const canvas = document.createElement('canvas');
    canvas.width = 200;
    canvas.height = 50;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) {
        throw new Error('Canvas context not available');
    }
    
    // Multi-layer rendering to detect spoofing
    ctx.textBaseline = 'top';
    ctx.font = '14px "Arial"';
    ctx.textBaseline = 'alphabetic';
    ctx.fillStyle = '#f60';
    ctx.fillRect(125, 1, 62, 20);
    ctx.fillStyle = '#069';
    ctx.fillText('Scrybe 🦉', 2, 15);
    ctx.fillStyle = 'rgba(102, 204, 0, 0.7)';
    ctx.fillText('Fingerprint', 4, 17);
    
    // Get image data
    const imageData = canvas.toDataURL();
    
    // Hash with SHA-256
    const hash = await hashString(imageData);
    
    return hash;
}

async function hashString(data: string): Promise<string> {
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}
```

## API Communication

### HMAC Authentication
```typescript
export class ScrybeClient {
    private readonly apiKey: string;
    private readonly hmacKey: CryptoKey;
    
    constructor(apiKey: string, private readonly baseUrl: string) {
        this.apiKey = apiKey;
    }
    
    async init(): Promise<void> {
        // Import HMAC key
        const encoder = new TextEncoder();
        const keyData = encoder.encode(this.apiKey);
        
        this.hmacKey = await crypto.subtle.importKey(
            'raw',
            keyData,
            { name: 'HMAC', hash: 'SHA-256' },
            false,
            ['sign']
        );
    }
    
    async sendTelemetry(data: BrowserTelemetry): Promise<void> {
        const timestamp = Date.now();
        const nonce = crypto.randomUUID();
        const body = JSON.stringify(data);
        
        // Generate HMAC signature
        const message = `${timestamp}:${nonce}:${body}`;
        const signature = await this.signMessage(message);
        
        // Send request
        const response = await fetch(`${this.baseUrl}/v1/ingest`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Scrybe-Timestamp': timestamp.toString(),
                'X-Scrybe-Nonce': nonce,
                'X-Scrybe-Signature': signature,
            },
            body,
        });
        
        if (!response.ok) {
            throw new Error(`API request failed: ${response.status}`);
        }
    }
    
    private async signMessage(message: string): Promise<string> {
        const encoder = new TextEncoder();
        const messageData = encoder.encode(message);
        
        const signatureBuffer = await crypto.subtle.sign(
            'HMAC',
            this.hmacKey,
            messageData
        );
        
        const signatureArray = Array.from(new Uint8Array(signatureBuffer));
        return signatureArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }
}
```

## GDPR Compliance

### Consent Management
```typescript
export interface ScrybeConfig {
    apiKey: string;
    apiUrl: string;
    consentGiven?: boolean;
    respectDoNotTrack?: boolean;
}

export class ScrybeSDK {
    private config: ScrybeConfig;
    private client: ScrybeClient;
    private isInitialized = false;
    
    constructor(config: ScrybeConfig) {
        this.config = config;
    }
    
    async init(): Promise<void> {
        // Check Do Not Track
        if (this.config.respectDoNotTrack && navigator.doNotTrack === '1') {
            console.info('[Scrybe] Do Not Track enabled. Fingerprinting disabled.');
            return;
        }
        
        // Check EU visitor (simplified heuristic)
        const isEU = this.detectEUVisitor();
        
        if (isEU && !this.config.consentGiven) {
            console.warn('[Scrybe] GDPR consent required. Call setConsent(true) after user consent.');
            return;
        }
        
        // Initialize client
        this.client = new ScrybeClient(this.config.apiKey, this.config.apiUrl);
        await this.client.init();
        
        // Start fingerprinting
        await this.startFingerprinting();
        
        this.isInitialized = true;
    }
    
    setConsent(granted: boolean): void {
        this.config.consentGiven = granted;
        
        if (granted && !this.isInitialized) {
            this.init();
        }
    }
    
    private detectEUVisitor(): boolean {
        // Simple timezone-based heuristic (not 100% accurate)
        const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
        const euTimezones = ['Europe/', 'GMT', 'UTC'];
        
        return euTimezones.some(tz => timezone.startsWith(tz));
    }
    
    private async startFingerprinting(): Promise<void> {
        const telemetry = await this.collectTelemetry();
        await this.client.sendTelemetry(telemetry);
    }
    
    private async collectTelemetry(): Promise<BrowserTelemetry> {
        const [canvasFp, webglFp] = await Promise.all([
            generateCanvasFingerprint(),
            generateWebGLFingerprint(),
        ]);
        
        return {
            userAgent: navigator.userAgent,
            screenWidth: window.screen.width,
            screenHeight: window.screen.height,
            colorDepth: window.screen.colorDepth,
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            language: navigator.language,
            plugins: Array.from(navigator.plugins).map(p => p.name),
            canvasFingerprint: canvasFp,
            webglFingerprint: webglFp,
        };
    }
}
```

## Error Handling

### Graceful Degradation
```typescript
export class ScrybeSDK {
    private async collectTelemetry(): Promise<BrowserTelemetry> {
        const basic: BrowserTelemetry = {
            userAgent: navigator.userAgent,
            screenWidth: window.screen.width,
            screenHeight: window.screen.height,
            colorDepth: window.screen.colorDepth,
            timezone: this.getTimezoneOrDefault(),
            language: navigator.language,
            plugins: this.getPluginsOrEmpty(),
        };
        
        // Try to collect fingerprints, fallback gracefully
        try {
            basic.canvasFingerprint = await generateCanvasFingerprint();
        } catch (error) {
            console.warn('[Scrybe] Canvas fingerprint failed:', error);
        }
        
        try {
            basic.webglFingerprint = await generateWebGLFingerprint();
        } catch (error) {
            console.warn('[Scrybe] WebGL fingerprint failed:', error);
        }
        
        return basic;
    }
    
    private getTimezoneOrDefault(): string {
        try {
            return Intl.DateTimeFormat().resolvedOptions().timeZone;
        } catch {
            return 'UTC';
        }
    }
    
    private getPluginsOrEmpty(): string[] {
        try {
            return Array.from(navigator.plugins).map(p => p.name);
        } catch {
            return [];
        }
    }
}
```

## Testing Standards

### Unit Tests with Jest
```typescript
// __tests__/bounded-array.test.ts

import { BoundedArray } from '../bounded-array';

describe('BoundedArray', () => {
    test('prevents exceeding capacity', () => {
        const arr = new BoundedArray<number>(3);
        
        expect(arr.push(1)).toBe(true);
        expect(arr.push(2)).toBe(true);
        expect(arr.push(3)).toBe(true);
        expect(arr.push(4)).toBe(false); // Should fail
        
        expect(arr.length).toBe(3);
    });
    
    test('throws on invalid capacity', () => {
        expect(() => new BoundedArray(0)).toThrow();
        expect(() => new BoundedArray(-1)).toThrow();
    });
});
```

## Build & Bundle Configuration

### Webpack Configuration
```javascript
// webpack.config.js
module.exports = {
    entry: './src/index.ts',
    output: {
        filename: 'scrybe-sdk.min.js',
        library: 'Scrybe',
        libraryTarget: 'umd',
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    optimization: {
        minimize: true,
    },
};
```

## Code Quality Checklist

Before committing JavaScript/TypeScript:
- [ ] TypeScript strict mode enabled
- [ ] No `any` types (use `unknown` if necessary)
- [ ] All collections bounded
- [ ] Error handling with try-catch
- [ ] GDPR consent checks
- [ ] Tests pass: `npm test`
- [ ] Linting passes: `npm run lint`
- [ ] Build succeeds: `npm run build`
- [ ] Bundle size < 50KB gzipped
