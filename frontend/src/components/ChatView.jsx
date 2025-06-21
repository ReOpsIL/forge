import React from 'react';
import Terminal from './Terminal';
import './ChatView.css';

const ChatView = () => {
    return (
        <div className="chat-view">
            <div className="chat-content">
                <Terminal />
            </div>
        </div>
    );
};

export default ChatView;