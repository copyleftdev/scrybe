/**
 * Core type definitions for Scrybe SDK
 * 
 * @module types
 */

/**
 * Network signals observable from the client side
 */
export interface NetworkSignals {
  /** Connection type (4g, wifi, etc.) */
  effectiveType?: string;
  /** Download speed in Mbps */
  downlink?: number;
  /** Round-trip time in ms */
  rtt?: number;
  /** HTTP version from Performance API */
  httpVersion?: string;
}

/**
 * Canvas fingerprint data
 */
export interface CanvasFingerprint {
  /** SHA-256 hash of canvas rendering */
  hash: string;
  /** Whether canvas is supported */
  supported: boolean;
}

/**
 * WebGL fingerprint data
 */
export interface WebGLFingerprint {
  /** SHA-256 hash of WebGL parameters */
  hash: string;
  /** GPU vendor */
  vendor: string;
  /** GPU renderer */
  renderer: string;
  /** Supported WebGL extensions */
  supportedExtensions: string[];
}

/**
 * Audio fingerprint data
 */
export interface AudioFingerprint {
  /** SHA-256 hash of audio context */
  hash: string;
  /** Whether audio context is supported */
  supported: boolean;
}

/**
 * Font detection result
 */
export interface FontFingerprint {
  /** List of detected fonts */
  available: string[];
  /** Hash of font list */
  hash: string;
}

/**
 * Screen and viewport information
 */
export interface ScreenInfo {
  width: number;
  height: number;
  colorDepth: number;
  pixelRatio: number;
  orientation: string;
}

/**
 * Navigator information
 */
export interface NavigatorInfo {
  userAgent: string;
  language: string;
  languages: string[];
  platform: string;
  hardwareConcurrency?: number;
  deviceMemory?: number;
  maxTouchPoints?: number;
}

/**
 * Browser automation detection
 */
export interface AutomationQuirks {
  /** navigator.webdriver flag */
  webdriver: boolean;
  /** Generic automation detection */
  automation: boolean;
  /** PhantomJS detection */
  phantom: boolean;
  /** Selenium detection */
  selenium: boolean;
}

/**
 * Storage availability
 */
export interface StorageInfo {
  localStorage: boolean;
  sessionStorage: boolean;
  indexedDB: boolean;
  cookies: boolean;
}

/**
 * Plugin information
 */
export interface PluginInfo {
  count: number;
  list: string[];
}

/**
 * Complete browser signals
 */
export interface BrowserSignals {
  canvas: CanvasFingerprint;
  webgl?: WebGLFingerprint;
  audio?: AudioFingerprint;
  fonts?: FontFingerprint;
  screen: ScreenInfo;
  navigator: NavigatorInfo;
  quirks: AutomationQuirks;
  storage: StorageInfo;
  plugins: PluginInfo;
}

/**
 * Mouse event data (privacy-safe)
 */
export interface MouseEvent {
  timestamp: number;
  x: number;
  y: number;
  type: 'move' | 'click' | 'scroll';
}

/**
 * Scroll event data
 */
export interface ScrollEvent {
  timestamp: number;
  position: number;
  velocity: number;
}

/**
 * Click event data
 */
export interface ClickEvent {
  timestamp: number;
  x: number;
  y: number;
  button: number;
}

/**
 * Keyboard activity (NO input values)
 */
export interface KeyboardActivity {
  eventCount: number;
  avgTimeBetweenKeys: number;
  hasActivity: boolean;
}

/**
 * Page timing information
 */
export interface TimingInfo {
  timeOnPage: number;
  idleTime: number;
  activeTime: number;
  focusChanges: number;
  visibilityChanges: number;
}

/**
 * User interaction metrics
 */
export interface InteractionInfo {
  scrollDepth: number;
  maxScrollVelocity: number;
  elementsClicked: number;
  formsInteracted: number;
}

/**
 * Behavioral signals from user interaction
 */
export interface BehavioralSignals {
  mouse: {
    events: MouseEvent[];
    entropy: number;
    velocity: number[];
    acceleration: number[];
  };
  scroll: {
    events: ScrollEvent[];
    velocity: number[];
    smoothness: number;
  };
  clicks: {
    events: ClickEvent[];
    density: number;
    timing: number[];
  };
  keyboard: KeyboardActivity;
  timing: TimingInfo;
  interaction: InteractionInfo;
}

/**
 * Complete telemetry payload
 */
export interface TelemetryPayload {
  sessionId: string;
  timestamp: number;
  network: NetworkSignals;
  browser: BrowserSignals;
  behavioral: BehavioralSignals;
}

/**
 * SDK configuration options
 */
export interface ScrybeConfig {
  /** API endpoint URL */
  apiUrl: string;
  /** API key for authentication */
  apiKey: string;
  /** Whether user has given consent (GDPR) */
  consentGiven?: boolean;
  /** Respect Do Not Track header */
  respectDoNotTrack?: boolean;
  /** Enable debug logging */
  debug?: boolean;
  /** Collection timeout in ms */
  timeout?: number;
}

/**
 * API response from ingestion endpoint
 */
export interface IngestResponse {
  sessionId: string;
  isNew: boolean;
  timestamp: string;
}
