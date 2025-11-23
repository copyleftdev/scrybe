import type { BehavioralSignals } from '../types';
export declare class BehavioralCollector {
    private mouseEvents;
    private scrollEvents;
    private clickEvents;
    private keyPressCount;
    private keyPressTimes;
    private startTime;
    private activeTime;
    private idleTime;
    private focusChangeCount;
    private visibilityChangeCount;
    private lastActivityTime;
    private maxScrollDepth;
    private elementsClickedCount;
    private formsInteractedCount;
    constructor();
    private attachListeners;
    private updateActivity;
    getSignals(): BehavioralSignals;
    private getMouseSignals;
    private getScrollSignals;
    private getClickSignals;
    private getKeyboardSignals;
    private getTimingInfo;
    private getInteractionInfo;
    private calculateVelocities;
    private calculateAccelerations;
    private calculateEntropy;
    private calculateSmoothness;
    private calculateInterClickIntervals;
    private calculateClickDensity;
}
//# sourceMappingURL=behavioral.d.ts.map