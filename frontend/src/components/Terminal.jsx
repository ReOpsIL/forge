import React, { useState, useEffect, useRef } from 'react';
import { InputText } from 'primereact/inputtext';
import { Button } from 'primereact/button';
import './Terminal.css';

const Terminal = () => {
    const [messages, setMessages] = useState([]);
    const [input, setInput] = useState('');
    const [isConnected, setIsConnected] = useState(false);
    const [isProcessing, setIsProcessing] = useState(false);
    const messagesEndRef = useRef(null);
    const websocketRef = useRef(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    };

    useEffect(() => {
        scrollToBottom();
    }, [messages]);

    useEffect(() => {
        connectWebSocket();
        return () => {
            if (websocketRef.current) {
                websocketRef.current.close();
            }
        };
    }, []);

    const connectWebSocket = () => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/chat/ws`;
        
        websocketRef.current = new WebSocket(wsUrl);

        websocketRef.current.onopen = () => {
            setIsConnected(true);
            addMessage('system', 'Connected to Claude terminal');
        };

        websocketRef.current.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'output') {
                addMessage('claude', data.content);
            } else if (data.type === 'error') {
                addMessage('error', data.content);
            } else if (data.type === 'processing') {
                setIsProcessing(data.processing);
            }
        };

        websocketRef.current.onclose = () => {
            setIsConnected(false);
            addMessage('system', 'Connection to Claude terminal closed');
        };

        websocketRef.current.onerror = (error) => {
            console.error('WebSocket error:', error);
            addMessage('error', 'WebSocket connection error');
        };
    };

    const addMessage = (type, content) => {
        setMessages(prev => [...prev, {
            id: Date.now() + Math.random(),
            type,
            content,
            timestamp: new Date()
        }]);
    };

    const sendMessage = () => {
        if (!input.trim() || !isConnected || isProcessing) return;

        const message = input.trim();
        addMessage('user', message);
        setInput('');
        setIsProcessing(true);

        if (websocketRef.current && websocketRef.current.readyState === WebSocket.OPEN) {
            websocketRef.current.send(JSON.stringify({
                type: 'input',
                content: message
            }));
        }
    };

    const handleKeyPress = (e) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    };

    const getMessageClass = (type) => {
        switch (type) {
            case 'user':
                return 'terminal-message user-message';
            case 'claude':
                return 'terminal-message claude-message';
            case 'system':
                return 'terminal-message system-message';
            case 'error':
                return 'terminal-message error-message';
            default:
                return 'terminal-message';
        }
    };

    const formatTimestamp = (timestamp) => {
        return timestamp.toLocaleTimeString();
    };

    return (
        <div className="terminal-container">
            <div className="terminal-header">
                <div className="terminal-status">
                    <span className={`status-indicator ${isConnected ? 'connected' : 'disconnected'}`}></span>
                    <span className="status-text">
                        {isConnected ? 'Connected' : 'Disconnected'}
                        {isProcessing && ' - Processing...'}
                    </span>
                </div>
                <Button
                    icon="pi pi-refresh"
                    className="p-button-text p-button-sm"
                    onClick={connectWebSocket}
                    disabled={isConnected}
                    tooltip="Reconnect"
                />
            </div>
            
            <div className="terminal-messages">
                {messages.map((message) => (
                    <div key={message.id} className={getMessageClass(message.type)}>
                        <div className="message-header">
                            <span className="message-type">
                                {message.type === 'user' ? '>' : 
                                 message.type === 'claude' ? 'Claude:' : 
                                 message.type === 'system' ? 'System:' : 'Error:'}
                            </span>
                            <span className="message-timestamp">
                                {formatTimestamp(message.timestamp)}
                            </span>
                        </div>
                        <div className="message-content">
                            {message.content}
                        </div>
                    </div>
                ))}
                <div ref={messagesEndRef} />
            </div>

            <div className="terminal-input">
                <InputText
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyPress={handleKeyPress}
                    placeholder="Type your message to Claude..."
                    disabled={!isConnected || isProcessing}
                    className="terminal-input-field"
                />
                <Button
                    icon="pi pi-send"
                    onClick={sendMessage}
                    disabled={!input.trim() || !isConnected || isProcessing}
                    className="terminal-send-button"
                    tooltip="Send message"
                />
            </div>
        </div>
    );
};

export default Terminal;