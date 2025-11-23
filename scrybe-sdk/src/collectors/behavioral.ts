/**
 * Behavioral signals collector
 * 
 * Collects user interaction patterns (mouse, scroll, keyboard) with strict
 * privacy protections - NO input values, only interaction patterns.
 */

import type {
  BehavioralSignals,
  MouseEvent,
  ScrollEvent,
  ClickEvent,
  KeyboardActivity,
  TimingInfo,
  InteractionInfo,
} from '../types';

/**
 * Maximum events to prevent DoS
 */
const MAX_MOUSE_EVENTS = 100;
const MAX_SCROLL_EVENTS = 50;
const MAX_CLICK_EVENTS = 20;

export class BehavioralCollector {
  private mouseEvents: MouseEvent[] = [];
  private scrollEvents: ScrollEvent[] = [];
  private clickEvents: ClickEvent[] = [];
  private keyPressCount = 0;
  private keyPressTimes: number[] = [];
  private startTime = Date.now();
  private activeTime = 0;
  private idleTime = 0;
  private focusChangeCount = 0;
  private visibilityChangeCount = 0;
  private lastActivityTime = Date.now();
  private maxScrollDepth = 0;
  private elementsClickedCount = 0;
  private formsInteractedCount = 0;

  constructor() {
    this.attachListeners();
  }

  /**
   * Attach event listeners for behavioral tracking
   * 
   * @private
   */
  private attachListeners(): void {
    // Mouse movement (throttled)
    let lastMouseSample = 0;
    const mouseSampleInterval = 100; // Sample every 100ms

    document.addEventListener(
      'mousemove',
      (e) => {
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
      },
      { passive: true }
    );

    // Click events
    document.addEventListener(
      'click',
      (e) => {
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
      },
      { passive: true }
    );

    // Scroll events (throttled)
    let lastScrollSample = 0;
    const scrollSampleInterval = 200;

    document.addEventListener(
      'scroll',
      () => {
        const now = Date.now();
        if (now - lastScrollSample >= scrollSampleInterval && this.scrollEvents.length < MAX_SCROLL_EVENTS) {
          const scrollY = window.scrollY || window.pageYOffset;
          const scrollHeight = document.documentElement.scrollHeight - window.innerHeight;
          const scrollDepth = scrollHeight > 0 ? (scrollY / scrollHeight) * 100 : 0;

          this.maxScrollDepth = Math.max(this.maxScrollDepth, scrollDepth);

          const velocity =
            this.scrollEvents.length > 0
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
      },
      { passive: true }
    );

    // Keyboard events (NO values, only count and timing)
    document.addEventListener(
      'keydown',
      () => {
        const now = Date.now();
        this.keyPressTimes.push(now);
        this.keyPressCount++;
        this.updateActivity();

        // Keep only last 50 timings
        if (this.keyPressTimes.length > 50) {
          this.keyPressTimes.shift();
        }
      },
      { passive: true }
    );

    // Form interaction (count only, NO values)
    document.addEventListener(
      'input',
      (e) => {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          this.formsInteractedCount++;
          this.updateActivity();
        }
      },
      { passive: true }
    );

    // Focus changes
    window.addEventListener('focus', () => {
      this.focusChangeCount++;
    });

    window.addEventListener('blur', () => {
      this.focusChangeCount++;
    });

    // Visibility changes
    document.addEventListener('visibilitychange', () => {
      this.visibilityChangeCount++;
    });
  }

  /**
   * Update activity tracking
   * 
   * @private
   */
  private updateActivity(): void {
    this.lastActivityTime = Date.now();
  }

  /**
   * Get collected behavioral signals
   * 
   * @returns Behavioral signals object
   */
  getSignals(): BehavioralSignals {
    return {
      mouse: this.getMouseSignals(),
      scroll: this.getScrollSignals(),
      clicks: this.getClickSignals(),
      keyboard: this.getKeyboardSignals(),
      timing: this.getTimingInfo(),
      interaction: this.getInteractionInfo(),
    };
  }

  /**
   * Get mouse movement signals
   * 
   * @private
   */
  private getMouseSignals() {
    const velocities = this.calculateVelocities(this.mouseEvents);
    const accelerations = this.calculateAccelerations(velocities);

    return {
      events: this.mouseEvents.slice(0, MAX_MOUSE_EVENTS),
      entropy: this.calculateEntropy(this.mouseEvents),
      velocity: velocities.slice(0, 50),
      acceleration: accelerations.slice(0, 50),
    };
  }

  /**
   * Get scroll signals
   * 
   * @private
   */
  private getScrollSignals() {
    const velocities = this.scrollEvents.map((e) => e.velocity);

    return {
      events: this.scrollEvents.slice(0, MAX_SCROLL_EVENTS),
      velocity: velocities.slice(0, 30),
      smoothness: this.calculateSmoothness(velocities),
    };
  }

  /**
   * Get click signals
   * 
   * @private
   */
  private getClickSignals() {
    const timings = this.calculateInterClickIntervals();

    return {
      events: this.clickEvents.slice(0, MAX_CLICK_EVENTS),
      density: this.calculateClickDensity(),
      timing: timings,
    };
  }

  /**
   * Get keyboard activity (NO input values)
   * 
   * @private
   */
  private getKeyboardSignals(): KeyboardActivity {
    const avgTime =
      this.keyPressTimes.length > 1
        ? this.keyPressTimes.slice(1).reduce((acc, time, i) => acc + (time - this.keyPressTimes[i]!), 0) /
          (this.keyPressTimes.length - 1)
        : 0;

    return {
      eventCount: this.keyPressCount,
      avgTimeBetweenKeys: avgTime,
      hasActivity: this.keyPressCount > 0,
    };
  }

  /**
   * Get timing information
   * 
   * @private
   */
  private getTimingInfo(): TimingInfo {
    const now = Date.now();
    const totalTime = now - this.startTime;
    const idleThreshold = 5000; // 5 seconds

    const idleTime = now - this.lastActivityTime > idleThreshold ? now - this.lastActivityTime : this.idleTime;

    return {
      timeOnPage: totalTime,
      idleTime,
      activeTime: totalTime - idleTime,
      focusChanges: this.focusChangeCount,
      visibilityChanges: this.visibilityChangeCount,
    };
  }

  /**
   * Get interaction metrics
   * 
   * @private
   */
  private getInteractionInfo(): InteractionInfo {
    const maxVelocity = Math.max(...this.scrollEvents.map((e) => e.velocity), 0);

    return {
      scrollDepth: this.maxScrollDepth,
      maxScrollVelocity: maxVelocity,
      elementsClicked: this.elementsClickedCount,
      formsInteracted: this.formsInteractedCount,
    };
  }

  /**
   * Calculate velocities from mouse events
   * 
   * @private
   */
  private calculateVelocities(events: MouseEvent[]): number[] {
    const velocities: number[] = [];

    for (let i = 1; i < events.length; i++) {
      const prev = events[i - 1]!;
      const curr = events[i]!;
      const dx = curr.x - prev.x;
      const dy = curr.y - prev.y;
      const dt = curr.timestamp - prev.timestamp;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const velocity = dt > 0 ? distance / dt : 0;
      velocities.push(velocity);
    }

    return velocities;
  }

  /**
   * Calculate accelerations from velocities
   * 
   * @private
   */
  private calculateAccelerations(velocities: number[]): number[] {
    const accelerations: number[] = [];

    for (let i = 1; i < velocities.length; i++) {
      const accel = velocities[i]! - velocities[i - 1]!;
      accelerations.push(accel);
    }

    return accelerations;
  }

  /**
   * Calculate Shannon entropy of mouse movements
   * 
   * @private
   */
  private calculateEntropy(events: MouseEvent[]): number {
    if (events.length === 0) return 0;

    // Simplified entropy calculation
    const positions = events.map((e) => `${Math.floor(e.x / 10)},${Math.floor(e.y / 10)}`);
    const frequency = new Map<string, number>();

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

  /**
   * Calculate smoothness of scroll
   * 
   * @private
   */
  private calculateSmoothness(velocities: number[]): number {
    if (velocities.length < 2) return 0;

    const variance =
      velocities.reduce((acc, v) => {
        const mean = velocities.reduce((a, b) => a + b, 0) / velocities.length;
        return acc + Math.pow(v - mean, 2);
      }, 0) / velocities.length;

    return 1 / (1 + variance); // Higher value = smoother
  }

  /**
   * Calculate inter-click intervals
   * 
   * @private
   */
  private calculateInterClickIntervals(): number[] {
    const intervals: number[] = [];

    for (let i = 1; i < this.clickEvents.length; i++) {
      const interval = this.clickEvents[i]!.timestamp - this.clickEvents[i - 1]!.timestamp;
      intervals.push(interval);
    }

    return intervals;
  }

  /**
   * Calculate click density (clicks per area)
   * 
   * @private
   */
  private calculateClickDensity(): number {
    if (this.clickEvents.length === 0) return 0;

    const area = window.innerWidth * window.innerHeight;
    return (this.clickEvents.length / area) * 1000000; // Normalize
  }
}
