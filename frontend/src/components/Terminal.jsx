import React, { useEffect, useRef, useState } from 'react'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import './Terminal.module.css'

// Singleton Terminal Manager
class TerminalManager {
    constructor() {
        this.xterm = null
        this.fitAddon = null
        this.ws = null
        this.subscribers = new Set()
        this.connectionStatus = 'disconnected'
        this.wsUrl = null
        this.reconnectAttempts = 0
        this.maxReconnectAttempts = 10
        this.reconnectDelay = 3000
        this.currentElement = null
        this.userScrolledUp = false
        this.lastScrollTop = 0
    }

    initialize(wsUrl) {
        if (this.xterm) return

        this.wsUrl = wsUrl

        // Create XTerm instance
        this.xterm = new XTerm({
            theme: {
                background: '#1e1e1e',
                foreground: '#ffffff',
                cursor: '#ffffff',
                selection: 'rgba(255, 255, 255, 0.3)',
                black: '#000000',
                red: '#ff5555',
                green: '#50fa7b',
                yellow: '#f1fa8c',
                blue: '#bd93f9',
                magenta: '#ff79c6',
                cyan: '#8be9fd',
                white: '#bfbfbf',
                brightBlack: '#4d4d4d',
                brightRed: '#ff6e67',
                brightGreen: '#5af78e',
                brightYellow: '#f4f99d',
                brightBlue: '#caa9fa',
                brightMagenta: '#ff92d0',
                brightCyan: '#9aedfe',
                brightWhite: '#e6e6e6'
            },
            fontSize: 14,
            fontFamily: 'Monaco, Menlo, "Ubuntu Mono", "Consolas", "DejaVu Sans Mono", monospace',
            cursorBlink: true,
            convertEol: false,
            scrollback: 1000,
            allowTransparency: false,
            disableStdin: false,
            cols: 120,
            rows: 30,
            lineHeight: 1.0,
            letterSpacing: 0,
            allowProposedApi: true
        })

        // Add fit addon
        this.fitAddon = new FitAddon()
        this.xterm.loadAddon(this.fitAddon)

        // Setup input handler
        this.xterm.onData((data) => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(data)
            }
        })

        // Setup scroll detection
        this.xterm.onScroll(() => {
            this.detectUserScroll()
        })

        // Connect WebSocket
        this.connectWebSocket()
    }

    connectWebSocket() {
        if (!this.wsUrl) return

        // Close existing connection
        if (this.ws) {
            this.ws.close()
        }

        this.updateConnectionStatus('connecting')

        try {
            this.ws = new WebSocket(this.wsUrl)

            this.ws.onopen = () => {
                console.log('WebSocket connected')
                this.reconnectAttempts = 0
                this.updateConnectionStatus('connected')
            }

            this.ws.onmessage = (event) => {
                if (this.xterm) {
                    const wasAtBottom = this.isAtBottom()
                    this.xterm.write(event.data)
                    
                    // Only auto-scroll if user was at bottom or hasn't manually scrolled up
                    if (wasAtBottom && !this.userScrolledUp) {
                        this.scrollToBottom()
                    }
                }
            }

            this.ws.onclose = (event) => {
                console.log('WebSocket closed:', event.code, event.reason)
                this.updateConnectionStatus('disconnected')
                this.ws = null
                
                // Auto-reconnect with exponential backoff
                if (this.reconnectAttempts < this.maxReconnectAttempts) {
                    const delay = this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts)
                    this.reconnectAttempts++
                    
                    setTimeout(() => {
                        if (this.xterm && this.connectionStatus !== 'connected') {
                            this.connectWebSocket()
                        }
                    }, delay)
                } else {
                    this.updateConnectionStatus('error')
                }
            }

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error)
                this.updateConnectionStatus('error')
            }
        } catch (error) {
            console.error('Failed to create WebSocket:', error)
            this.updateConnectionStatus('error')
        }
    }

    updateConnectionStatus(status) {
        this.connectionStatus = status
        this.notifySubscribers()
    }

    subscribe(callback) {
        this.subscribers.add(callback)
        return () => this.subscribers.delete(callback)
    }

    notifySubscribers() {
        this.subscribers.forEach(callback => callback(this.connectionStatus))
    }

    attachToElement(element) {
        if (!this.xterm || !element) return

        // Only attach if not already attached to this element
        if (this.currentElement === element) return
        
        this.currentElement = element
        
        // Safely attach to new element
        try {
            this.xterm.open(element)
        } catch (error) {
            // If already attached somewhere, create a new element wrapper
            console.warn('Terminal already attached, reattaching...', error)
            // Clear the element and try again
            element.innerHTML = ''
            this.xterm.open(element)
        }
        
        // Fit and focus
        setTimeout(() => {
            try {
                this.fitAddon.fit()
                this.xterm.focus()
            } catch (error) {
                console.warn('Failed to fit terminal:', error)
            }
        }, 100)
    }

    detach() {
        // Keep track of current element but don't actually detach
        // XTerm will handle the DOM changes
        this.currentElement = null
    }

    fit() {
        if (this.fitAddon && this.xterm && this.currentElement) {
            try {
                // Force a reflow before fitting
                setTimeout(() => {
                    // Get the container dimensions
                    const rect = this.currentElement.getBoundingClientRect()
                    const proposed = this.fitAddon.proposeDimensions()
                    
                    if (proposed && proposed.rows > 0) {
                        // Use a large fixed column count to prevent wrapping
                        // This ensures text will be cropped rather than wrapped
                        const fixedCols = 200  // Large enough to prevent most wrapping
                        
                        // Resize the terminal with fixed wide columns
                        this.xterm.resize(fixedCols, proposed.rows)
                        
                        // Notify backend about terminal size change
                        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                            const dims = { cols: fixedCols, rows: proposed.rows }
                            // Send resize notification to backend if needed
                            // this.ws.send(JSON.stringify({type: 'resize', cols: dims.cols, rows: dims.rows}))
                        }
                    }
                }, 10)
            } catch (error) {
                console.warn('Failed to resize terminal:', error)
            }
        }
    }

    focus() {
        if (this.xterm) {
            this.xterm.focus()
        }
    }

    reconnect() {
        this.reconnectAttempts = 0
        this.connectWebSocket()
    }

    getConnectionStatus() {
        return this.connectionStatus
    }

    detectUserScroll() {
        if (!this.xterm) return
        
        const currentScrollTop = this.xterm.buffer.active.viewportY
        const maxScrollTop = this.xterm.buffer.active.length - this.xterm.rows
        
        // Check if user scrolled up from bottom
        if (currentScrollTop < maxScrollTop - 1) {
            this.userScrolledUp = true
        } else {
            this.userScrolledUp = false
        }
        
        this.lastScrollTop = currentScrollTop
    }

    isAtBottom() {
        if (!this.xterm) return true
        
        const currentScrollTop = this.xterm.buffer.active.viewportY
        const maxScrollTop = this.xterm.buffer.active.length - this.xterm.rows
        
        return currentScrollTop >= maxScrollTop - 1
    }

    scrollToBottom() {
        if (!this.xterm) return
        
        this.xterm.scrollToBottom()
        this.userScrolledUp = false
    }

    injectText(text) {
        if (this.xterm && this.ws && this.ws.readyState === WebSocket.OPEN) {
            // Send the text to the terminal
            this.ws.send(text)
            // // Simulate pressing Enter to execute the command
            this.ws.send('\r')
        }
    }
}

// Global singleton instance
const terminalManager = new TerminalManager()

// Export globally for access from other components
if (typeof window !== 'undefined') {
    window.terminalManager = terminalManager
}

const Terminal = ({ 
    title = 'Claude Terminal',
    wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/claude/ws`
}) => {
    const terminalRef = useRef(null)
    const [connectionStatus, setConnectionStatus] = useState('disconnected')

    useEffect(() => {
        // Initialize terminal manager
        terminalManager.initialize(wsUrl)

        // Subscribe to status updates
        const unsubscribe = terminalManager.subscribe(setConnectionStatus)

        // Set initial status
        setConnectionStatus(terminalManager.getConnectionStatus())

        return unsubscribe
    }, [wsUrl])

    useEffect(() => {
        if (terminalRef.current) {
            // Attach terminal to current element
            terminalManager.attachToElement(terminalRef.current)

            // Handle window resize
            const handleResize = () => {
                terminalManager.fit()
            }
            window.addEventListener('resize', handleResize)

            return () => {
                window.removeEventListener('resize', handleResize)
                // Don't detach the terminal, just clean up the reference
                terminalManager.detach()
            }
        }
    }, [])

    const getStatusColor = () => {
        switch (connectionStatus) {
            case 'connected': return '#50fa7b'
            case 'connecting': return '#f1fa8c'
            case 'disconnected': return '#ff6e67'
            case 'error': return '#ff5555'
            default: return '#6272a4'
        }
    }

    const handleReconnect = () => {
        terminalManager.reconnect()
    }

    const handleTerminalClick = () => {
        terminalManager.focus()
    }

    return (
        <div className="terminal-container">
            <div className="terminal-header">
                <div className="terminal-title">{title}</div>
                <div className="terminal-controls">
                    <div className="connection-status">
                        <div 
                            className="status-indicator"
                            style={{ backgroundColor: getStatusColor() }}
                        />
                        <span className="status-text">{connectionStatus}</span>
                    </div>
                    {connectionStatus !== 'connected' && (
                        <button 
                            className="reconnect-button"
                            onClick={handleReconnect}
                            disabled={connectionStatus === 'connecting'}
                        >
                            {connectionStatus === 'connecting' ? 'Connecting...' : 'Reconnect'}
                        </button>
                    )}
                </div>
            </div>
            <div 
                className="terminal-body"
                ref={terminalRef}
                onClick={handleTerminalClick}
            />
        </div>
    )
}

export default Terminal