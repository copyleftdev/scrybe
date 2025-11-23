export interface NetworkSignals {
    effectiveType?: string;
    downlink?: number;
    rtt?: number;
    httpVersion?: string;
}
export interface CanvasFingerprint {
    hash: string;
    supported: boolean;
}
export interface WebGLFingerprint {
    hash: string;
    vendor: string;
    renderer: string;
    supportedExtensions: string[];
}
export interface AudioFingerprint {
    hash: string;
    supported: boolean;
}
export interface FontFingerprint {
    available: string[];
    hash: string;
}
export interface ScreenInfo {
    width: number;
    height: number;
    colorDepth: number;
    pixelRatio: number;
    orientation: string;
}
export interface NavigatorInfo {
    userAgent: string;
    language: string;
    languages: string[];
    platform: string;
    hardwareConcurrency?: number;
    deviceMemory?: number;
    maxTouchPoints?: number;
}
export interface AutomationQuirks {
    webdriver: boolean;
    automation: boolean;
    phantom: boolean;
    selenium: boolean;
}
export interface StorageInfo {
    localStorage: boolean;
    sessionStorage: boolean;
    indexedDB: boolean;
    cookies: boolean;
}
export interface PluginInfo {
    count: number;
    list: string[];
}
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
export interface MouseEvent {
    timestamp: number;
    x: number;
    y: number;
    type: 'move' | 'click' | 'scroll';
}
export interface ScrollEvent {
    timestamp: number;
    position: number;
    velocity: number;
}
export interface ClickEvent {
    timestamp: number;
    x: number;
    y: number;
    button: number;
}
export interface KeyboardActivity {
    eventCount: number;
    avgTimeBetweenKeys: number;
    hasActivity: boolean;
}
export interface TimingInfo {
    timeOnPage: number;
    idleTime: number;
    activeTime: number;
    focusChanges: number;
    visibilityChanges: number;
}
export interface InteractionInfo {
    scrollDepth: number;
    maxScrollVelocity: number;
    elementsClicked: number;
    formsInteracted: number;
}
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
export interface TelemetryPayload {
    sessionId: string;
    timestamp: number;
    network: NetworkSignals;
    browser: BrowserSignals;
    behavioral: BehavioralSignals;
}
export interface ScrybeConfig {
    apiUrl: string;
    apiKey: string;
    consentGiven?: boolean;
    respectDoNotTrack?: boolean;
    debug?: boolean;
    timeout?: number;
}
export interface IngestResponse {
    sessionId: string;
    isNew: boolean;
    timestamp: string;
}
//# sourceMappingURL=types.d.ts.map