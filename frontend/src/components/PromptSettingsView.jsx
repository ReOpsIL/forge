import React, { useState, useEffect, useRef } from 'react';
import { Card } from 'primereact/card';
import { Button } from 'primereact/button';
import { Toast } from 'primereact/toast';
import { TabMenu } from 'primereact/tabmenu';
import { Editor } from '@monaco-editor/react';
import { Tooltip } from 'primereact/tooltip';
import './ProjectView.css';

const PromptSettingsView = ({ setActiveView }) => {
    const [projectConfig, setProjectConfig] = useState({
        // User-configurable prompts
        auto_complete_system_prompt: '',
        auto_complete_user_prompt: '',
        enhance_description_system_prompt: '',
        enhance_description_user_prompt: '',
        generate_tasks_system_prompt: '',
        generate_tasks_user_prompt: '',
        process_markdown_spec_system_prompt: '',
        process_markdown_spec_user_prompt: ''
    });
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [activePromptIndex, setActivePromptIndex] = useState(0);

    // Create a ref for the toast
    const toastRef = useRef(null);

    // Define the prompt types for the tabs
    const promptTabs = [
        { label: 'Auto Complete', icon: 'pi pi-pencil' },
        { label: 'Enhance Description', icon: 'pi pi-file-edit' },
        { label: 'Generate Tasks', icon: 'pi pi-list' },
        { label: 'Process Markdown', icon: 'pi pi-file' }
    ];

    useEffect(() => {
        fetchProjectConfig();
    }, []);

    const fetchProjectConfig = async () => {
        try {
            setLoading(true);
            const response = await fetch('/api/project');

            if (response.status === 404) {
                // If the project config doesn't exist yet, use default values
                setProjectConfig({
                    auto_complete_system_prompt: '',
                    auto_complete_user_prompt: '',
                    enhance_description_system_prompt: '',
                    enhance_description_user_prompt: '',
                    generate_tasks_system_prompt: '',
                    generate_tasks_user_prompt: '',
                    process_markdown_spec_system_prompt: '',
                    process_markdown_spec_user_prompt: ''
                });
                setLoading(false);
                return;
            }

            if (!response.ok) {
                throw new Error('Failed to fetch project configuration');
            }

            const data = await response.json();
            setProjectConfig(data);
        } catch (error) {
            console.error('Error fetching project configuration:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to load project configuration',
                life: 3000
            });
        } finally {
            setLoading(false);
        }
    };

    const saveProjectConfig = async () => {
        try {
            setSaving(true);
            const response = await fetch('/api/project', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(projectConfig),
            });

            if (!response.ok) {
                throw new Error('Failed to save project configuration');
            }

            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Prompt settings saved successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error saving project configuration:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to save prompt settings',
                life: 3000
            });
        } finally {
            setSaving(false);
        }
    };

    const handleInputChange = (field, value) => {
        setProjectConfig({
            ...projectConfig,
            [field]: value
        });
    };

    const renderPromptEditor = () => {
        switch (activePromptIndex) {
            case 0: // Auto Complete
                return (
                    <>
                        <div className="field">
                            <label htmlFor="auto_complete_system_prompt">
                                System Prompt
                                <Tooltip target=".auto-complete-system-help" position="right">
                                    System prompt for auto-completing block descriptions.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 auto-complete-system-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.auto_complete_system_prompt}
                                    onChange={(value) => handleInputChange('auto_complete_system_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="auto_complete_user_prompt">
                                User Prompt
                                <Tooltip target=".auto-complete-user-help" position="right">
                                    User prompt template for auto-completing block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 auto-complete-user-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.auto_complete_user_prompt}
                                    onChange={(value) => handleInputChange('auto_complete_user_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                    </>
                );
            case 1: // Enhance Description
                return (
                    <>
                        <div className="field">
                            <label htmlFor="enhance_description_system_prompt">
                                System Prompt
                                <Tooltip target=".enhance-system-help" position="right">
                                    System prompt for enhancing block descriptions.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 enhance-system-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.enhance_description_system_prompt}
                                    onChange={(value) => handleInputChange('enhance_description_system_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="enhance_description_user_prompt">
                                User Prompt
                                <Tooltip target=".enhance-user-help" position="right">
                                    User prompt template for enhancing block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 enhance-user-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.enhance_description_user_prompt}
                                    onChange={(value) => handleInputChange('enhance_description_user_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                    </>
                );
            case 2: // Generate Tasks
                return (
                    <>
                        <div className="field">
                            <label htmlFor="generate_tasks_system_prompt">
                                System Prompt
                                <Tooltip target=".tasks-system-help" position="right">
                                    System prompt for generating tasks from block descriptions.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 tasks-system-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.generate_tasks_system_prompt}
                                    onChange={(value) => handleInputChange('generate_tasks_system_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="generate_tasks_user_prompt">
                                User Prompt
                                <Tooltip target=".tasks-user-help" position="right">
                                    User prompt template for generating tasks from block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 tasks-user-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.generate_tasks_user_prompt}
                                    onChange={(value) => handleInputChange('generate_tasks_user_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                    </>
                );
            case 3: // Process Markdown
                return (
                    <>
                        <div className="field">
                            <label htmlFor="process_markdown_spec_system_prompt">
                                System Prompt
                                <Tooltip target=".markdown-system-help" position="right">
                                    System prompt for processing markdown specifications.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-system-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_markdown_spec_system_prompt}
                                    onChange={(value) => handleInputChange('process_markdown_spec_system_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="process_markdown_spec_user_prompt">
                                User Prompt
                                <Tooltip target=".markdown-user-help" position="right">
                                    User prompt template for processing markdown specifications. Use {} as a placeholder for the markdown content.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-user-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_markdown_spec_user_prompt}
                                    onChange={(value) => handleInputChange('process_markdown_spec_user_prompt', value || '')}
                                    options={{
                                        minimap: { enabled: false },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        lineNumbers: 'on',
                                        automaticLayout: true
                                    }}
                                />
                            </div>
                        </div>
                    </>
                );
            default:
                return null;
        }
    };

    if (loading) {
        return <div>Loading prompt settings...</div>;
    }

    return (
        <div className="project-container">
            <Toast ref={toastRef} />

            <div className="flex justify-content-between align-items-center mb-3">
                <div className="flex align-items-center">
                    <Button
                        icon="pi pi-arrow-left"
                        className="p-button-text mr-2"
                        onClick={() => setActiveView('project')}
                        tooltip="Back to Project Settings"
                        tooltipOptions={{ position: 'bottom' }}
                    />
                    <h2>Prompt Settings</h2>
                </div>
                <Button
                    label="Save"
                    icon="pi pi-save"
                    className="p-button-success"
                    onClick={saveProjectConfig}
                    loading={saving}
                />
            </div>

            <TabMenu 
                model={promptTabs} 
                activeIndex={activePromptIndex} 
                onTabChange={(e) => setActivePromptIndex(e.index)} 
                className="mb-3"
            />

            <Card className="project-card">
                <div className="p-fluid">
                    {renderPromptEditor()}
                </div>
            </Card>
        </div>
    );
};

export default PromptSettingsView;
