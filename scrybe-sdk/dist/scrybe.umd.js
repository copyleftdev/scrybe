(function (global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports) :
    typeof define === 'function' && define.amd ? define(['exports'], factory) :
    (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.Scrybe = {}));
})(this, (function (exports) { 'use strict';

    class StaticCollector {
        async collect() {
            return {
                network: this.collectNetworkSignals(),
                browser: {
                    screen: this.collectScreenInfo(),
                    navigator: this.collectNavigatorInfo(),
                    quirks: this.detectAutomation(),
                    storage: this.checkStorage(),
                    plugins: this.collectPlugins(),
                },
            };
        }
        collectNetworkSignals() {
            const connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection;
            return {
                effectiveType: connection?.effectiveType,
                downlink: connection?.downlink,
                rtt: connection?.rtt,
                httpVersion: this.getHttpVersion(),
            };
        }
        getHttpVersion() {
            try {
                const entry = performance.getEntriesByType('navigation')[0];
                return entry?.nextHopProtocol;
            }
            catch {
                return undefined;
            }
        }
        collectScreenInfo() {
            return {
                width: screen.width,
                height: screen.height,
                colorDepth: screen.colorDepth,
                pixelRatio: window.devicePixelRatio,
                orientation: screen.orientation?.type || 'unknown',
            };
        }
        collectNavigatorInfo() {
            return {
                userAgent: navigator.userAgent,
                language: navigator.language,
                languages: Array.from(navigator.languages || [navigator.language]),
                platform: navigator.platform,
                hardwareConcurrency: navigator.hardwareConcurrency,
                deviceMemory: navigator.deviceMemory,
                maxTouchPoints: navigator.maxTouchPoints,
            };
        }
        detectAutomation() {
            return {
                webdriver: navigator.webdriver === true,
                automation: this.detectGenericAutomation(),
                phantom: this.detectPhantomJS(),
                selenium: this.detectSelenium(),
            };
        }
        detectGenericAutomation() {
            const win = window;
            return !!(win._phantom ||
                win.__nightmare ||
                win._selenium ||
                win.callPhantom ||
                win.callSelenium ||
                win.__webdriver_evaluate ||
                win.__driver_evaluate ||
                win.__webdriver_script_function ||
                win.__webdriver_script_func ||
                win.__webdriver_script_fn ||
                win.__fxdriver_evaluate ||
                win.__driver_unwrapped ||
                win.__webdriver_unwrapped ||
                win.__selenium_unwrapped);
        }
        detectPhantomJS() {
            const win = window;
            return !!(win._phantom || win.callPhantom);
        }
        detectSelenium() {
            const win = window;
            const doc = document;
            return !!(win._selenium ||
                win.__selenium_unwrapped ||
                win.__webdriver_evaluate ||
                win.__driver_evaluate ||
                doc.__webdriver_evaluate ||
                doc.__selenium_unwrapped ||
                doc.__driver_unwrapped);
        }
        checkStorage() {
            return {
                localStorage: this.isStorageAvailable('localStorage'),
                sessionStorage: this.isStorageAvailable('sessionStorage'),
                indexedDB: 'indexedDB' in window,
                cookies: navigator.cookieEnabled,
            };
        }
        isStorageAvailable(type) {
            try {
                const storage = window[type];
                const test = '__storage_test__';
                storage.setItem(test, test);
                storage.removeItem(test);
                return true;
            }
            catch {
                return false;
            }
        }
        collectPlugins() {
            const plugins = Array.from(navigator.plugins || []);
            return {
                count: plugins.length,
                list: plugins.map((p) => p.name).slice(0, 20),
            };
        }
    }

    async function sha256(data) {
        const encoder = new TextEncoder();
        const dataBuffer = encoder.encode(data);
        const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
    }

    class CanvasCollector {
        async collect() {
            try {
                const canvas = this.createCanvas();
                const ctx = canvas.getContext('2d');
                if (!ctx) {
                    return {
                        hash: '',
                        supported: false,
                    };
                }
                this.renderPattern(ctx);
                const dataUrl = canvas.toDataURL();
                const hash = await sha256(dataUrl);
                return {
                    hash,
                    supported: true,
                };
            }
            catch (error) {
                return {
                    hash: '',
                    supported: false,
                };
            }
        }
        createCanvas() {
            const canvas = document.createElement('canvas');
            canvas.width = 200;
            canvas.height = 50;
            return canvas;
        }
        renderPattern(ctx) {
            ctx.textBaseline = 'top';
            const gradient = ctx.createLinearGradient(0, 0, 200, 50);
            gradient.addColorStop(0, '#f60');
            gradient.addColorStop(0.5, '#069');
            gradient.addColorStop(1, '#0f0');
            ctx.fillStyle = gradient;
            ctx.fillRect(0, 0, 200, 50);
            ctx.font = '14px "Arial"';
            ctx.fillStyle = '#f60';
            ctx.fillRect(125, 1, 62, 20);
            ctx.fillStyle = '#069';
            ctx.fillText('Scrybe 🦉', 2, 15);
            ctx.fillStyle = 'rgba(102, 204, 0, 0.7)';
            ctx.fillText('Bot Detection', 4, 17);
            ctx.beginPath();
            ctx.arc(50, 25, 20, 0, Math.PI * 2, true);
            ctx.closePath();
            ctx.strokeStyle = '#ff0000';
            ctx.stroke();
            ctx.font = '12px "Courier New"';
            ctx.fillStyle = '#000';
            ctx.fillText('FP Test', 120, 30);
        }
    }

    class WebGLCollector {
        async collect() {
            try {
                const canvas = document.createElement('canvas');
                const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
                if (!gl) {
                    return undefined;
                }
                const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
                const vendor = debugInfo
                    ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL)
                    : 'unknown';
                const renderer = debugInfo
                    ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL)
                    : 'unknown';
                const extensions = gl.getSupportedExtensions() || [];
                const parameters = this.collectParameters(gl);
                const renderHash = await this.renderPattern(gl, canvas);
                const combined = JSON.stringify({
                    vendor,
                    renderer,
                    extensions: extensions.sort(),
                    parameters,
                    renderHash,
                });
                const hash = await sha256(combined);
                return {
                    hash,
                    vendor,
                    renderer,
                    supportedExtensions: extensions.slice(0, 20),
                };
            }
            catch (error) {
                return undefined;
            }
        }
        collectParameters(gl) {
            const params = {};
            const keys = [
                'VERSION',
                'SHADING_LANGUAGE_VERSION',
                'VENDOR',
                'RENDERER',
                'MAX_TEXTURE_SIZE',
                'MAX_VIEWPORT_DIMS',
                'MAX_VERTEX_ATTRIBS',
                'MAX_VERTEX_UNIFORM_VECTORS',
                'MAX_VARYING_VECTORS',
                'MAX_COMBINED_TEXTURE_IMAGE_UNITS',
                'MAX_VERTEX_TEXTURE_IMAGE_UNITS',
                'MAX_TEXTURE_IMAGE_UNITS',
                'MAX_FRAGMENT_UNIFORM_VECTORS',
                'ALIASED_LINE_WIDTH_RANGE',
                'ALIASED_POINT_SIZE_RANGE',
            ];
            for (const key of keys) {
                try {
                    const value = gl.getParameter(gl[key]);
                    if (value !== null) {
                        params[key] = Array.isArray(value) ? Array.from(value) : value;
                    }
                }
                catch {
                }
            }
            return params;
        }
        async renderPattern(gl, canvas) {
            const vertexShader = gl.createShader(gl.VERTEX_SHADER);
            const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
            if (!vertexShader || !fragmentShader) {
                return 'no_render';
            }
            gl.shaderSource(vertexShader, `
      attribute vec2 position;
      void main() {
        gl_Position = vec4(position, 0.0, 1.0);
      }
    `);
            gl.compileShader(vertexShader);
            gl.shaderSource(fragmentShader, `
      precision mediump float;
      void main() {
        gl_FragColor = vec4(1.0, 0.0, 0.5, 1.0);
      }
    `);
            gl.compileShader(fragmentShader);
            const program = gl.createProgram();
            if (!program) {
                return 'no_render';
            }
            gl.attachShader(program, vertexShader);
            gl.attachShader(program, fragmentShader);
            gl.linkProgram(program);
            gl.useProgram(program);
            const vertices = new Float32Array([-0.5, -0.5, 0.5, -0.5, 0.0, 0.5]);
            const buffer = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
            gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
            const position = gl.getAttribLocation(program, 'position');
            gl.enableVertexAttribArray(position);
            gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
            gl.clearColor(0, 0, 0, 1);
            gl.clear(gl.COLOR_BUFFER_BIT);
            gl.drawArrays(gl.TRIANGLES, 0, 3);
            const pixels = new Uint8Array(canvas.width * canvas.height * 4);
            gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
            const sample = Array.from(pixels.slice(0, 100)).join(',');
            return await sha256(sample);
        }
    }

    class AudioCollector {
        async collect() {
            try {
                const AudioContext = window.AudioContext || window.webkitAudioContext;
                if (!AudioContext) {
                    return undefined;
                }
                const context = new AudioContext();
                const oscillator = context.createOscillator();
                const analyser = context.createAnalyser();
                const gainNode = context.createGain();
                const scriptProcessor = context.createScriptProcessor(4096, 1, 1);
                gainNode.gain.value = 0;
                oscillator.type = 'triangle';
                oscillator.frequency.value = 10000;
                oscillator.connect(analyser);
                analyser.connect(scriptProcessor);
                scriptProcessor.connect(gainNode);
                gainNode.connect(context.destination);
                const fingerprint = await new Promise((resolve) => {
                    scriptProcessor.onaudioprocess = (event) => {
                        const output = event.outputBuffer.getChannelData(0);
                        const fingerprint = this.computeFingerprint(output);
                        resolve(fingerprint);
                    };
                    oscillator.start(0);
                    setTimeout(() => {
                        oscillator.stop();
                    }, 100);
                });
                context.close();
                const hash = await sha256(fingerprint);
                return {
                    hash,
                    supported: true,
                };
            }
            catch (error) {
                return {
                    hash: '',
                    supported: false,
                };
            }
        }
        computeFingerprint(data) {
            const samples = [];
            const sampleInterval = Math.floor(data.length / 30);
            for (let i = 0; i < data.length; i += sampleInterval) {
                samples.push(Math.abs(data[i] || 0));
            }
            return samples.map((s) => s.toFixed(10)).join(',');
        }
    }

    class FontCollector {
        constructor() {
            this.testFonts = [
                'Arial', 'Verdana', 'Courier New', 'Times New Roman', 'Georgia',
                'Palatino Linotype', 'Book Antiqua', 'Comic Sans MS', 'Impact',
                'Lucida Console', 'Tahoma', 'Trebuchet MS', 'Arial Black',
                'Monaco', 'Menlo', 'Helvetica Neue', 'Geneva', 'Apple Chancery',
                'Baskerville', 'Big Caslon', 'Copperplate', 'Didot', 'Futura',
                'Gill Sans', 'Optima', 'Palatino',
                'Liberation Sans', 'Liberation Serif', 'Liberation Mono',
                'DejaVu Sans', 'DejaVu Serif', 'DejaVu Sans Mono',
                'Ubuntu', 'Cantarell', 'Droid Sans', 'Noto Sans',
                'Roboto', 'Open Sans', 'Lato', 'Montserrat', 'Source Sans Pro',
            ];
            this.baseFonts = ['monospace', 'sans-serif', 'serif'];
            this.testString = 'mmmmmmmmmmlli';
            this.testSize = '72px';
        }
        async collect() {
            try {
                const availableFonts = this.detectFonts();
                if (availableFonts.length === 0) {
                    return undefined;
                }
                availableFonts.sort();
                const combined = availableFonts.join('|');
                const hash = await sha256(combined);
                return {
                    available: availableFonts,
                    hash,
                };
            }
            catch (error) {
                return undefined;
            }
        }
        detectFonts() {
            const canvas = document.createElement('canvas');
            const context = canvas.getContext('2d');
            if (!context) {
                return [];
            }
            const baseMeasurements = new Map();
            for (const baseFont of this.baseFonts) {
                context.font = `${this.testSize} ${baseFont}`;
                const metrics = context.measureText(this.testString);
                baseMeasurements.set(baseFont, {
                    width: metrics.width,
                    height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent,
                });
            }
            const detected = [];
            for (const testFont of this.testFonts) {
                let hasDifference = false;
                for (const baseFont of this.baseFonts) {
                    context.font = `${this.testSize} '${testFont}', ${baseFont}`;
                    const metrics = context.measureText(this.testString);
                    const measurement = {
                        width: metrics.width,
                        height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent,
                    };
                    const baseline = baseMeasurements.get(baseFont);
                    if (!baseline)
                        continue;
                    if (Math.abs(measurement.width - baseline.width) > 0.5 ||
                        Math.abs(measurement.height - baseline.height) > 0.5) {
                        hasDifference = true;
                        break;
                    }
                }
                if (hasDifference) {
                    detected.push(testFont);
                }
            }
            return detected;
        }
    }

    const MAX_MOUSE_EVENTS = 100;
    const MAX_SCROLL_EVENTS = 50;
    const MAX_CLICK_EVENTS = 20;
    class BehavioralCollector {
        constructor() {
            this.mouseEvents = [];
            this.scrollEvents = [];
            this.clickEvents = [];
            this.keyPressCount = 0;
            this.keyPressTimes = [];
            this.startTime = Date.now();
            this.activeTime = 0;
            this.idleTime = 0;
            this.focusChangeCount = 0;
            this.visibilityChangeCount = 0;
            this.lastActivityTime = Date.now();
            this.maxScrollDepth = 0;
            this.elementsClickedCount = 0;
            this.formsInteractedCount = 0;
            this.attachListeners();
        }
        attachListeners() {
            let lastMouseSample = 0;
            const mouseSampleInterval = 100;
            document.addEventListener('mousemove', (e) => {
                const now = Date.now();
                if (now - lastMouseSample >= mouseSampleInterval && this.mouseEvents.length < MAX_MOUSE_EVENTS) {
                    this.mouseEvents.push({
                        timestamp: now,
                        x: e.clientX,
                        y: e.clientY,
                        type: 'move',
                    });
                    lastMouseSample = now;
                    this.updateActivity();
                }
            }, { passive: true });
            document.addEventListener('click', (e) => {
                if (this.clickEvents.length < MAX_CLICK_EVENTS) {
                    this.clickEvents.push({
                        timestamp: Date.now(),
                        x: e.clientX,
                        y: e.clientY,
                        button: e.button,
                    });
                    this.elementsClickedCount++;
                    this.updateActivity();
                }
            }, { passive: true });
            let lastScrollSample = 0;
            const scrollSampleInterval = 200;
            document.addEventListener('scroll', () => {
                const now = Date.now();
                if (now - lastScrollSample >= scrollSampleInterval && this.scrollEvents.length < MAX_SCROLL_EVENTS) {
                    const scrollY = window.scrollY || window.pageYOffset;
                    const scrollHeight = document.documentElement.scrollHeight - window.innerHeight;
                    const scrollDepth = scrollHeight > 0 ? (scrollY / scrollHeight) * 100 : 0;
                    this.maxScrollDepth = Math.max(this.maxScrollDepth, scrollDepth);
                    const velocity = this.scrollEvents.length > 0
                        ? Math.abs(scrollY - (this.scrollEvents[this.scrollEvents.length - 1]?.position || 0)) /
                            (now - (this.scrollEvents[this.scrollEvents.length - 1]?.timestamp || now))
                        : 0;
                    this.scrollEvents.push({
                        timestamp: now,
                        position: scrollY,
                        velocity,
                    });
                    lastScrollSample = now;
                    this.updateActivity();
                }
            }, { passive: true });
            document.addEventListener('keydown', () => {
                const now = Date.now();
                this.keyPressTimes.push(now);
                this.keyPressCount++;
                this.updateActivity();
                if (this.keyPressTimes.length > 50) {
                    this.keyPressTimes.shift();
                }
            }, { passive: true });
            document.addEventListener('input', (e) => {
                const target = e.target;
                if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
                    this.formsInteractedCount++;
                    this.updateActivity();
                }
            }, { passive: true });
            window.addEventListener('focus', () => {
                this.focusChangeCount++;
            });
            window.addEventListener('blur', () => {
                this.focusChangeCount++;
            });
            document.addEventListener('visibilitychange', () => {
                this.visibilityChangeCount++;
            });
        }
        updateActivity() {
            this.lastActivityTime = Date.now();
        }
        getSignals() {
            return {
                mouse: this.getMouseSignals(),
                scroll: this.getScrollSignals(),
                clicks: this.getClickSignals(),
                keyboard: this.getKeyboardSignals(),
                timing: this.getTimingInfo(),
                interaction: this.getInteractionInfo(),
            };
        }
        getMouseSignals() {
            const velocities = this.calculateVelocities(this.mouseEvents);
            const accelerations = this.calculateAccelerations(velocities);
            return {
                events: this.mouseEvents.slice(0, MAX_MOUSE_EVENTS),
                entropy: this.calculateEntropy(this.mouseEvents),
                velocity: velocities.slice(0, 50),
                acceleration: accelerations.slice(0, 50),
            };
        }
        getScrollSignals() {
            const velocities = this.scrollEvents.map((e) => e.velocity);
            return {
                events: this.scrollEvents.slice(0, MAX_SCROLL_EVENTS),
                velocity: velocities.slice(0, 30),
                smoothness: this.calculateSmoothness(velocities),
            };
        }
        getClickSignals() {
            const timings = this.calculateInterClickIntervals();
            return {
                events: this.clickEvents.slice(0, MAX_CLICK_EVENTS),
                density: this.calculateClickDensity(),
                timing: timings,
            };
        }
        getKeyboardSignals() {
            const avgTime = this.keyPressTimes.length > 1
                ? this.keyPressTimes.slice(1).reduce((acc, time, i) => acc + (time - this.keyPressTimes[i]), 0) /
                    (this.keyPressTimes.length - 1)
                : 0;
            return {
                eventCount: this.keyPressCount,
                avgTimeBetweenKeys: avgTime,
                hasActivity: this.keyPressCount > 0,
            };
        }
        getTimingInfo() {
            const now = Date.now();
            const totalTime = now - this.startTime;
            const idleThreshold = 5000;
            const idleTime = now - this.lastActivityTime > idleThreshold ? now - this.lastActivityTime : this.idleTime;
            return {
                timeOnPage: totalTime,
                idleTime,
                activeTime: totalTime - idleTime,
                focusChanges: this.focusChangeCount,
                visibilityChanges: this.visibilityChangeCount,
            };
        }
        getInteractionInfo() {
            const maxVelocity = Math.max(...this.scrollEvents.map((e) => e.velocity), 0);
            return {
                scrollDepth: this.maxScrollDepth,
                maxScrollVelocity: maxVelocity,
                elementsClicked: this.elementsClickedCount,
                formsInteracted: this.formsInteractedCount,
            };
        }
        calculateVelocities(events) {
            const velocities = [];
            for (let i = 1; i < events.length; i++) {
                const prev = events[i - 1];
                const curr = events[i];
                const dx = curr.x - prev.x;
                const dy = curr.y - prev.y;
                const dt = curr.timestamp - prev.timestamp;
                const distance = Math.sqrt(dx * dx + dy * dy);
                const velocity = dt > 0 ? distance / dt : 0;
                velocities.push(velocity);
            }
            return velocities;
        }
        calculateAccelerations(velocities) {
            const accelerations = [];
            for (let i = 1; i < velocities.length; i++) {
                const accel = velocities[i] - velocities[i - 1];
                accelerations.push(accel);
            }
            return accelerations;
        }
        calculateEntropy(events) {
            if (events.length === 0)
                return 0;
            const positions = events.map((e) => `${Math.floor(e.x / 10)},${Math.floor(e.y / 10)}`);
            const frequency = new Map();
            for (const pos of positions) {
                frequency.set(pos, (frequency.get(pos) || 0) + 1);
            }
            let entropy = 0;
            const total = positions.length;
            for (const count of frequency.values()) {
                const p = count / total;
                entropy -= p * Math.log2(p);
            }
            return entropy;
        }
        calculateSmoothness(velocities) {
            if (velocities.length < 2)
                return 0;
            const variance = velocities.reduce((acc, v) => {
                const mean = velocities.reduce((a, b) => a + b, 0) / velocities.length;
                return acc + Math.pow(v - mean, 2);
            }, 0) / velocities.length;
            return 1 / (1 + variance);
        }
        calculateInterClickIntervals() {
            const intervals = [];
            for (let i = 1; i < this.clickEvents.length; i++) {
                const interval = this.clickEvents[i].timestamp - this.clickEvents[i - 1].timestamp;
                intervals.push(interval);
            }
            return intervals;
        }
        calculateClickDensity() {
            if (this.clickEvents.length === 0)
                return 0;
            const area = window.innerWidth * window.innerHeight;
            return (this.clickEvents.length / area) * 1000000;
        }
    }

    function generateNonce() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
            const r = (Math.random() * 16) | 0;
            const v = c === 'x' ? r : (r & 0x3) | 0x8;
            return v.toString(16);
        });
    }
    async function signPayload(body, timestamp, nonce, apiKey) {
        const message = `${timestamp}:${nonce}:${body}`;
        const keyData = new TextEncoder().encode(apiKey);
        const key = await crypto.subtle.importKey('raw', keyData, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
        const messageData = new TextEncoder().encode(message);
        const signature = await crypto.subtle.sign('HMAC', key, messageData);
        const hashArray = Array.from(new Uint8Array(signature));
        return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
    }

    class HttpClient {
        constructor(config) {
            this.config = config;
        }
        async send(payload) {
            const timestamp = Date.now();
            const nonce = generateNonce();
            const body = JSON.stringify(payload);
            const signature = await signPayload(body, timestamp, nonce, this.config.apiKey);
            const response = await fetch(`${this.config.apiUrl}/api/v1/ingest`, {
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
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            return response.json();
        }
    }

    class ConsentManager {
        constructor(config) {
            this.config = config;
            this.consentGiven = config.consentGiven || false;
        }
        hasConsent() {
            return this.consentGiven;
        }
        setConsent(granted) {
            this.consentGiven = granted;
            try {
                localStorage.setItem('scrybe_consent', granted ? '1' : '0');
            }
            catch {
            }
        }
        isEUVisitor() {
            try {
                const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
                const euTimezones = ['Europe/', 'GMT', 'UTC', 'WET', 'CET', 'EET'];
                return euTimezones.some((tz) => timezone.startsWith(tz));
            }
            catch {
                return false;
            }
        }
        getStoredConsent() {
            try {
                const stored = localStorage.getItem('scrybe_consent');
                if (stored === '1')
                    return true;
                if (stored === '0')
                    return false;
                return null;
            }
            catch {
                return null;
            }
        }
    }

    function generateSessionId() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
            const r = (Math.random() * 16) | 0;
            const v = c === 'x' ? r : (r & 0x3) | 0x8;
            return v.toString(16);
        });
    }

    class Scrybe {
        constructor(config) {
            this.isInitialized = false;
            this.config = {
                respectDoNotTrack: true,
                debug: false,
                timeout: 5000,
                ...config,
            };
            this.sessionId = generateSessionId();
            this.httpClient = new HttpClient(this.config);
            this.consentManager = new ConsentManager(this.config);
        }
        async init() {
            if (this.config.respectDoNotTrack && this.isDNTEnabled()) {
                this.log('Do Not Track enabled. Skipping initialization.');
                return;
            }
            if (!this.consentManager.hasConsent()) {
                this.log('Consent not given. Skipping initialization.');
                return;
            }
            this.log('Initializing Scrybe SDK...');
            try {
                const payload = await this.collectSignals();
                await this.sendTelemetry(payload);
                this.isInitialized = true;
                this.log('Scrybe SDK initialized successfully');
            }
            catch (error) {
                this.log('Failed to initialize Scrybe SDK:', error);
                throw error;
            }
        }
        async collectSignals() {
            const staticCollector = new StaticCollector();
            const canvasCollector = new CanvasCollector();
            const webglCollector = new WebGLCollector();
            const audioCollector = new AudioCollector();
            const fontCollector = new FontCollector();
            const behavioralCollector = new BehavioralCollector();
            const [staticSignals, canvasFingerprint, webglFingerprint, audioFingerprint, fontFingerprint] = await Promise.all([
                staticCollector.collect(),
                canvasCollector.collect(),
                webglCollector.collect(),
                audioCollector.collect(),
                fontCollector.collect(),
            ]);
            const behavioralSignals = behavioralCollector.getSignals();
            return {
                sessionId: this.sessionId,
                timestamp: Date.now(),
                network: staticSignals.network,
                browser: {
                    ...staticSignals.browser,
                    canvas: canvasFingerprint,
                    webgl: webglFingerprint,
                    audio: audioFingerprint,
                    fonts: fontFingerprint,
                },
                behavioral: behavioralSignals,
            };
        }
        async sendTelemetry(payload) {
            return this.httpClient.send(payload);
        }
        isDNTEnabled() {
            return (navigator.doNotTrack === '1' ||
                window.doNotTrack === '1' ||
                navigator.msDoNotTrack === '1');
        }
        log(...args) {
            if (this.config.debug) {
                console.log('[Scrybe]', ...args);
            }
        }
        setConsent(granted) {
            this.consentManager.setConsent(granted);
            if (granted && !this.isInitialized) {
                this.init().catch((err) => this.log('Initialization failed:', err));
            }
        }
        getSessionId() {
            return this.sessionId;
        }
    }

    exports.Scrybe = Scrybe;
    exports.default = Scrybe;

    Object.defineProperty(exports, '__esModule', { value: true });

}));
//# sourceMappingURL=scrybe.umd.js.map
