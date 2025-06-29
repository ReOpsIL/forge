import React, { useEffect, useRef, useState } from 'react';
import { Dialog } from 'primereact/dialog';
import { Button } from 'primereact/button';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import './LogsDialog.css';

const LogsDialog = ({ visible, onHide, taskId, blockId }) => {
    const terminalRef = useRef(null);
    const xtermRef = useRef(null);
    const fitAddonRef = useRef(null);
    const [logs, setLogs] = useState('');
    const [loading, setLoading] = useState(false);

    // Initialize xterm.js terminal with same config as Terminal.jsx
    useEffect(() => {
        if (visible && terminalRef.current && !xtermRef.current) {
            // Create XTerm instance with same theme as Terminal.jsx
            xtermRef.current = new XTerm({
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
                cursorBlink: false,
                convertEol: true,
                scrollback: 1000,
                allowTransparency: false,
                disableStdin: true, // Read-only for logs
                cols: 120,
                rows: 30,
                lineHeight: 1.0,
                letterSpacing: 0,
                allowProposedApi: true
            });

            // Add fit addon
            fitAddonRef.current = new FitAddon();
            xtermRef.current.loadAddon(fitAddonRef.current);

            // Open terminal in the container
            xtermRef.current.open(terminalRef.current);
            
            // Fit terminal to container
            setTimeout(() => {
                if (fitAddonRef.current) {
                    fitAddonRef.current.fit();
                }
            }, 100);
        }

        return () => {
            if (xtermRef.current) {
                xtermRef.current.dispose();
                xtermRef.current = null;
                fitAddonRef.current = null;
            }
        };
    }, [visible]);

    // Fetch logs when dialog opens or taskId changes
    useEffect(() => {
        if (visible && taskId && blockId) {
            fetchLogs();
        }
    }, [visible, taskId, blockId]);

    // Update terminal content when logs change
    useEffect(() => {
        if (xtermRef.current && logs) {
            // Clear terminal and write logs
            xtermRef.current.clear();
            xtermRef.current.write(logs);
        }
    }, [logs]);

    const fetchLogs = async () => {
        if (!taskId || !blockId) return;

        setLoading(true);
        try {
            // Use the log task ID format that matches the backend
            const logTaskId = `${blockId}:${taskId}`;
            
            // Use EventSource for proper SSE handling
            const eventSource = new EventSource(`/api/logs/stream/${logTaskId}`);
            let logContent = '';
            let timeoutId;

            // Set a timeout to stop listening after 5 seconds
            timeoutId = setTimeout(() => {
                eventSource.close();
                setLogs(logContent || 'No logs available for this task.');
                setLoading(false);
            }, 5000);

            eventSource.onmessage = (event) => {
                const data = event.data;
                if (data === 'keep-alive') {
                    return; // Ignore keep-alive messages
                }
                logContent += data + '\n';
            };

            eventSource.onerror = () => {
                clearTimeout(timeoutId);
                eventSource.close();
                setLogs(logContent || 'Error loading logs or no logs available.');
                setLoading(false);
            };

            eventSource.onopen = () => {
                // Connection opened successfully
                console.log('Log stream connection opened for task:', logTaskId);
            };

        } catch (error) {
            console.error('Error fetching logs:', error);
            setLogs('Error loading logs.');
            setLoading(false);
        }
    };


    const handleDialogHide = () => {
        // Clean up terminal when dialog closes
        if (xtermRef.current) {
            xtermRef.current.dispose();
            xtermRef.current = null;
            fitAddonRef.current = null;
        }
        setLogs('');
        onHide();
    };

    const handleResize = () => {
        if (fitAddonRef.current && xtermRef.current) {
            setTimeout(() => {
                fitAddonRef.current.fit();
            }, 100);
        }
    };

    return (
        <Dialog
            header={`Task Logs - ${taskId}`}
            visible={visible}
            style={{ width: '80vw', height: '70vh' }}
            onHide={handleDialogHide}
            onShow={handleResize}
            maximizable
            footer={
                <div>
                    <Button
                        label="Close"
                        icon="pi pi-times"
                        className="p-button-text"
                        onClick={handleDialogHide}
                    />
                </div>
            }
        >
            <div className="logs-dialog-content">
                {loading && (
                    <div className="logs-loading">
                        <i className="pi pi-spin pi-spinner" style={{ fontSize: '2rem', marginRight: '0.5rem' }}></i>
                        <span>Loading logs...</span>
                    </div>
                )}
                <div 
                    className="logs-terminal-container"
                    ref={terminalRef}
                    style={{ 
                        width: '100%', 
                        height: '500px',
                        display: loading ? 'none' : 'block'
                    }}
                />
            </div>
        </Dialog>
    );
};

export default LogsDialog;