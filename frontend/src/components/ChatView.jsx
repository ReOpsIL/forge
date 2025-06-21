import React from 'react';
import Terminal from './Terminal';
import './ChatView.css';

const ChatView = () => {
    return (
        <div className="chat-view">
            <div className="chat-header">
                <h2>Claude Chat Terminal</h2>
                <p>Interactive chat with Claude AI assistant</p>
            </div>
            <div className="chat-content">
                <Terminal />
            </div>
        </div>
    );
};

export default ChatView;