import React, { useState, useEffect, useRef } from 'react';
import { Card } from 'primereact/card';
import { Button } from 'primereact/button';
import { Dropdown } from 'primereact/dropdown';
import { ScrollPanel } from 'primereact/scrollpanel';
import { ProgressSpinner } from 'primereact/progressspinner';
import { Checkbox } from 'primereact/checkbox';
import './LoggerView.css';

/**
 * LoggerView component for displaying real-time logs from Claude execution
 */
const LoggerView = () => {
    const [taskIds, setTaskIds] = useState([]);
    const [selectedTaskId, setSelectedTaskId] = useState(null);
    const [logs, setLogs] = useState([]);
    const [loading, setLoading] = useState(false);
    const [connected, setConnected] = useState(false);
    const [currentExecutingTask, setCurrentExecutingTask] = useState(null);
    const [currentBlockName, setCurrentBlockName] = useState('');
    const [liveMode, setLiveMode] = useState(true);
    const eventSourceRef = useRef(null);
    const scrollPanelRef = useRef(null);
    const pollingIntervalRef = useRef(null);
    const liveModeRef = useRef(liveMode);

    // Fetch available task IDs on component mount and start polling for current task
    useEffect(() => {
        fetchTaskIds();
        startPollingCurrentTask();
        
        // Cleanup function to close EventSource connection and stop polling when component unmounts
        return () => {
            if (eventSourceRef.current) {
                eventSourceRef.current.close();
            }
            if (pollingIntervalRef.current) {
                clearInterval(pollingIntervalRef.current);
            }
        };
    }, []);

    // Update ref when liveMode changes and restart polling
    useEffect(() => {
        liveModeRef.current = liveMode;
        if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
        }
        if (liveMode) {
            startPollingCurrentTask();
        }
    }, [liveMode]);

    // Auto-scroll to bottom when logs update
    useEffect(() => {
        if (scrollPanelRef.current) {
            const scrollPanel = scrollPanelRef.current.getElement();
            scrollPanel.scrollTop = scrollPanel.scrollHeight;
        }
    }, [logs]);

    // Fetch available task IDs
    const fetchTaskIds = async () => {
        try {
            const response = await fetch('/api/logs/tasks');
            if (!response.ok) {
                throw new Error('Failed to fetch task IDs');
            }
            
            const data = await response.json();
            setTaskIds(data.map(id => ({ label: id, value: id })));
        } catch (error) {
            console.error('Error fetching task IDs:', error);
        }
    };

    // Start polling for currently executing task
    const startPollingCurrentTask = () => {
        const pollCurrentTask = async () => {
            try {
                const response = await fetch('/api/blocks');
                if (!response.ok) {
                    throw new Error('Failed to fetch blocks');
                }
                
                const blocks = await response.json();
                let foundExecutingTask = null;
                let foundBlockName = '';
                
                // Find the first task with [IN-PROGRESS] status
                for (const block of blocks) {
                    if (block.todo_list) {
                        for (const [taskId, task] of Object.entries(block.todo_list)) {
                            if (task.status === '[IN-PROGRESS]') {
                                foundExecutingTask = `${block.block_id}:${taskId}`;
                                foundBlockName = block.title || block.block_id;
                                break;
                            }
                        }
                    }
                    if (foundExecutingTask) break;
                }
                
                setCurrentExecutingTask(foundExecutingTask);
                setCurrentBlockName(foundBlockName);
                
                // Auto-switch to the executing task if live mode is enabled and it's different from current selection
                if (liveModeRef.current && foundExecutingTask && foundExecutingTask !== selectedTaskId) {
                    setSelectedTaskId(foundExecutingTask);
                    connectToLogStream(foundExecutingTask);
                }
                
            } catch (error) {
                console.error('Error polling current task:', error);
            }
        };
        
        // Poll immediately and then every 2 seconds
        pollCurrentTask();
        pollingIntervalRef.current = setInterval(pollCurrentTask, 2000);
    };

    // Connect to the log stream for a specific task
    const connectToLogStream = (taskId) => {
        // Close any existing connection
        if (eventSourceRef.current) {
            eventSourceRef.current.close();
        }

        setLogs([]);
        setLoading(true);
        setConnected(false);

        // Create a new EventSource connection
        const eventSource = new EventSource(`/api/logs/stream/${taskId}`);
        
        // Handle connection open
        eventSource.onopen = () => {
            setConnected(true);
            setLoading(false);
        };
        
        // Handle incoming messages
        eventSource.onmessage = (event) => {
            if (event.data === 'keep-alive') {
                return; // Ignore keep-alive messages
            }
            
            setLogs(prevLogs => [...prevLogs, event.data]);
        };
        
        // Handle errors
        eventSource.onerror = (error) => {
            console.error('EventSource error:', error);
            setConnected(false);
            setLoading(false);
            eventSource.close();
        };
        
        eventSourceRef.current = eventSource;
    };

    // Handle task selection change
    const handleTaskChange = (e) => {
        const taskId = e.value;
        setSelectedTaskId(taskId);
        
        if (taskId) {
            connectToLogStream(taskId);
        }
    };

    // Refresh task IDs
    const handleRefresh = () => {
        fetchTaskIds();
    };

    // Disconnect from the log stream
    const handleDisconnect = () => {
        if (eventSourceRef.current) {
            eventSourceRef.current.close();
            setConnected(false);
        }
    };

    // Render the logs
    const renderLogs = () => {
        if (loading) {
            return (
                <div className="flex justify-content-center align-items-center" style={{ height: '300px' }}>
                    <ProgressSpinner />
                </div>
            );
        }
        
        if (!selectedTaskId) {
            return (
                <div className="flex justify-content-center align-items-center" style={{ height: '300px' }}>
                    <p>Select a task to view logs</p>
                </div>
            );
        }
        
        if (logs.length === 0) {
            return (
                <div className="flex justify-content-center align-items-center" style={{ height: '300px' }}>
                    <p>No logs available for this task</p>
                </div>
            );
        }
        
        return (
            <ScrollPanel ref={scrollPanelRef} style={{ width: '100%', height: '500px' }} className="custom-scrollpanel">
                <div className="log-container">
                    {logs.map((log, index) => (
                        <div key={index} className="log-line">
                            {log}
                        </div>
                    ))}
                </div>
            </ScrollPanel>
        );
    };

    return (
        <div className="logger-view">
            <Card title="Claude Execution Logger" className="mb-4">
                <div className="p-fluid">
                    <div className="flex mb-3 align-items-center">
                        <div className="flex align-items-center mr-3">
                            <Checkbox
                                inputId="liveMode"
                                checked={liveMode}
                                onChange={(e) => setLiveMode(e.checked)}
                                className="mr-2"
                            />
                            <label htmlFor="liveMode" className="live-mode-label">Live</label>
                        </div>
                        <Dropdown
                            value={selectedTaskId}
                            options={taskIds}
                            onChange={handleTaskChange}
                            placeholder="Select a task"
                            className="mr-2 flex-grow-1"
                            disabled={liveMode}
                        />
                        <Button
                            icon="pi pi-refresh"
                            onClick={handleRefresh}
                            className="mr-2"
                            tooltip="Refresh task list"
                        />
                        <Button
                            icon="pi pi-times"
                            onClick={handleDisconnect}
                            className="p-button-danger"
                            disabled={!connected}
                            tooltip="Disconnect"
                        />
                    </div>
                    
                    <div className="status-indicator mb-2">
                        {connected ? (
                            <span className="connected">Connected</span>
                        ) : (
                            <span className="disconnected">Disconnected</span>
                        )}
                        {currentExecutingTask && (
                            <span className="executing-task ml-3">
                                Currently executing: <strong>{currentBlockName}</strong> 
                                ({currentExecutingTask})
                            </span>
                        )}
                    </div>
                    
                    {renderLogs()}
                </div>
            </Card>
        </div>
    );
};

export default LoggerView;