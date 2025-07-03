import {EventEmitter} from 'events';
import {TerminalSession, terminalSessionManager} from '../services/terminal-session-manager';

export interface TerminalState {
    sessionId: string | null;
    isConnected: boolean;
    isConnecting: boolean;
    history: string[];
    currentInput: string;
    error: string | null;
    metadata: Record<string, any>;
}

export interface MultiTerminalState {
    sessions: Map<string, TerminalState>;
    activeSessionId: string | null;
    globalHistory: string[];
}

export class TerminalStore extends EventEmitter {
    private state: MultiTerminalState = {
        sessions: new Map(),
        activeSessionId: null,
        globalHistory: []
    };

    constructor() {
        super();

        terminalSessionManager.on('sessionCreated', (sessionId: string) => {
            this.initializeSessionState(sessionId);
        });

        terminalSessionManager.on('sessionCleaned', (sessionId: string) => {
            this.removeSessionState(sessionId);
        });
    }

    createSession(): string {
        const sessionId = terminalSessionManager.createSession();
        this.initializeSessionState(sessionId);

        if (!this.state.activeSessionId) {
            this.setActiveSession(sessionId);
        }

        this.emit('sessionCreated', sessionId);
        return sessionId;
    }

    getSession(sessionId: string): TerminalSession | undefined {
        return terminalSessionManager.getSession(sessionId);
    }

    cleanupSession(sessionId: string): void {
        terminalSessionManager.cleanupSession(sessionId);

        if (this.state.activeSessionId === sessionId) {
            const remainingSessions = Array.from(this.state.sessions.keys());
            const nextSession = remainingSessions.find(id => id !== sessionId);
            this.setActiveSession(nextSession || null);
        }

        this.emit('sessionCleaned', sessionId);
    }

    getAllSessions(): TerminalSession[] {
        return terminalSessionManager.getAllSessions();
    }

    getActiveSessions(): TerminalSession[] {
        return terminalSessionManager.getActiveSessions();
    }

    getActiveSessionId(): string | null {
        return this.state.activeSessionId;
    }

    setActiveSession(sessionId: string | null): void {
        if (sessionId && !this.state.sessions.has(sessionId)) {
            throw new Error(`Session ${sessionId} does not exist`);
        }

        const previousSessionId = this.state.activeSessionId;
        this.state.activeSessionId = sessionId;

        this.emit('activeSessionChanged', sessionId, previousSessionId);
    }

    getSessionState(sessionId: string): TerminalState | undefined {
        return this.state.sessions.get(sessionId);
    }

    getActiveSessionState(): TerminalState | undefined {
        if (!this.state.activeSessionId) {
            return undefined;
        }
        return this.state.sessions.get(this.state.activeSessionId);
    }

    updateSessionConnection(sessionId: string, isConnected: boolean, isConnecting: boolean = false): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.isConnected = isConnected;
            sessionState.isConnecting = isConnecting;
            sessionState.error = isConnected ? null : sessionState.error;

            this.emit('sessionConnectionChanged', sessionId, isConnected, isConnecting);
        }
    }

    addToHistory(sessionId: string, command: string): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.history.push(command);

            if (sessionState.history.length > 1000) {
                sessionState.history = sessionState.history.slice(-1000);
            }

            this.state.globalHistory.push(`[${sessionId}] ${command}`);
            if (this.state.globalHistory.length > 2000) {
                this.state.globalHistory = this.state.globalHistory.slice(-2000);
            }

            terminalSessionManager.updateSessionActivity(sessionId);
            this.emit('historyUpdated', sessionId, command);
        }
    }

    getHistory(sessionId: string): string[] {
        const sessionState = this.state.sessions.get(sessionId);
        return sessionState ? [...sessionState.history] : [];
    }

    getGlobalHistory(): string[] {
        return [...this.state.globalHistory];
    }

    setCurrentInput(sessionId: string, input: string): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.currentInput = input;
            this.emit('inputChanged', sessionId, input);
        }
    }

    getCurrentInput(sessionId: string): string {
        const sessionState = this.state.sessions.get(sessionId);
        return sessionState ? sessionState.currentInput : '';
    }

    setError(sessionId: string, error: string | null): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.error = error;
            this.emit('errorChanged', sessionId, error);
        }
    }

    getError(sessionId: string): string | null {
        const sessionState = this.state.sessions.get(sessionId);
        return sessionState ? sessionState.error : null;
    }

    setSessionMetadata(sessionId: string, key: string, value: any): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.metadata[key] = value;
            terminalSessionManager.setSessionMetadata(sessionId, key, value);
            this.emit('metadataChanged', sessionId, key, value);
        }
    }

    getSessionMetadata(sessionId: string, key?: string): any {
        const sessionState = this.state.sessions.get(sessionId);
        if (!sessionState) {
            return undefined;
        }

        return key ? sessionState.metadata[key] : {...sessionState.metadata};
    }

    clearHistory(sessionId: string): void {
        const sessionState = this.state.sessions.get(sessionId);
        if (sessionState) {
            sessionState.history = [];
            this.emit('historyCleared', sessionId);
        }
    }

    clearAllHistory(): void {
        this.state.globalHistory = [];
        for (const [sessionId, sessionState] of this.state.sessions) {
            sessionState.history = [];
            this.emit('historyCleared', sessionId);
        }
        this.emit('globalHistoryCleared');
    }

    getSessionCount(): number {
        return this.state.sessions.size;
    }

    getSessionIds(): string[] {
        return Array.from(this.state.sessions.keys());
    }

    switchToNextSession(): string | null {
        const sessionIds = this.getSessionIds();
        if (sessionIds.length === 0) {
            return null;
        }

        const currentIndex = this.state.activeSessionId
            ? sessionIds.indexOf(this.state.activeSessionId)
            : -1;

        const nextIndex = (currentIndex + 1) % sessionIds.length;
        const nextSessionId = sessionIds[nextIndex];

        this.setActiveSession(nextSessionId);
        return nextSessionId;
    }

    switchToPreviousSession(): string | null {
        const sessionIds = this.getSessionIds();
        if (sessionIds.length === 0) {
            return null;
        }

        const currentIndex = this.state.activeSessionId
            ? sessionIds.indexOf(this.state.activeSessionId)
            : 0;

        const prevIndex = currentIndex === 0 ? sessionIds.length - 1 : currentIndex - 1;
        const prevSessionId = sessionIds[prevIndex];

        this.setActiveSession(prevSessionId);
        return prevSessionId;
    }

    getState(): MultiTerminalState {
        return {
            sessions: new Map(this.state.sessions),
            activeSessionId: this.state.activeSessionId,
            globalHistory: [...this.state.globalHistory]
        };
    }

    getStats(): {
        totalSessions: number;
        connectedSessions: number;
        disconnectedSessions: number;
        totalHistoryEntries: number;
        globalHistoryEntries: number;
        activeSession: string | null;
    } {
        const connectedSessions = Array.from(this.state.sessions.values()).filter(s => s.isConnected);
        const totalHistoryEntries = Array.from(this.state.sessions.values())
            .reduce((sum, s) => sum + s.history.length, 0);

        return {
            totalSessions: this.state.sessions.size,
            connectedSessions: connectedSessions.length,
            disconnectedSessions: this.state.sessions.size - connectedSessions.length,
            totalHistoryEntries,
            globalHistoryEntries: this.state.globalHistory.length,
            activeSession: this.state.activeSessionId
        };
    }

    shutdown(): void {
        console.log('Shutting down terminal store...');

        const sessionIds = Array.from(this.state.sessions.keys());
        for (const sessionId of sessionIds) {
            this.cleanupSession(sessionId);
        }

        this.state.sessions.clear();
        this.state.activeSessionId = null;
        this.state.globalHistory = [];

        this.emit('shutdown');
        console.log('Terminal store shutdown complete');
    }

    private initializeSessionState(sessionId: string): void {
        const initialState: TerminalState = {
            sessionId,
            isConnected: false,
            isConnecting: false,
            history: [],
            currentInput: '',
            error: null,
            metadata: {}
        };

        this.state.sessions.set(sessionId, initialState);
    }

    private removeSessionState(sessionId: string): void {
        this.state.sessions.delete(sessionId);

        if (this.state.activeSessionId === sessionId) {
            const remainingSessions = Array.from(this.state.sessions.keys());
            this.state.activeSessionId = remainingSessions.length > 0 ? remainingSessions[0] : null;
        }
    }
}

export const terminalStore = new TerminalStore();