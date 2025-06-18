import React, {useState, useEffect, useRef} from 'react';
import {Card} from 'primereact/card';
import {InputText} from 'primereact/inputtext';
import {Button} from 'primereact/button';
import {Toast} from 'primereact/toast';
import {Editor} from '@monaco-editor/react';
import {Dropdown} from 'primereact/dropdown';
import './ProjectView.css';

const ProjectView = () => {
    const [projectConfig, setProjectConfig] = useState({
        git_repository_url: '',
        project_home_directory: '',
        project_description: '',
        llm_provider: '',
        openrouter_model: '',
        gemini_model: '',

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
    const [testingConnection, setTestingConnection] = useState(false);

    // LLM provider options
    const llmProviderOptions = [
        { label: 'OpenRouter', value: 'OpenRouter' },
        { label: 'Gemini', value: 'Gemini' }
    ];

    // OpenRouter model options
    const openrouterModelOptions = [
        { label: 'Gemini 2.5 Pro',        value: 'google/gemini-2.5-pro' },
        { label: 'Gemini 2.5 Flash',      value: 'google/gemini-2.5-flash' },
        { label: 'Gemini 2.5 Flash‑Lite (preview)', value: 'google/gemini-2.5-flash-lite-preview' },
        { label: 'Claude 4 Opus',        value: 'anthropic/claude-opus-4-20250514' },
        { label: 'Claude 4 Sonnet',      value: 'anthropic/claude-sonnet-4-20250514' },
        { label: 'GPT‑4o',               value: 'openai/gpt-4o' },
        { label: 'GPT‑4 Turbo',          value: 'openai/gpt-4-turbo' },
        { label: 'Mistral Large',        value: 'mistralai/mistral-large-latest' }
    ];



    // Gemini model options
    const geminiModelOptions = [
        { label: 'Gemini 2.5 Pro',                  value: 'gemini-2.5-pro' },
        { label: 'Gemini 2.5 Flash',                value: 'gemini-2.5-flash' },
        { label: 'Gemini 2.5 Flash‑Lite (preview)', value: 'gemini-2.5-flash-lite-preview' },
        { label: 'Gemini Ultra',                    value: 'gemini-ultra' }
    ];

    // Create a ref for the toast
    const toastRef = useRef(null);

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
                    git_repository_url: '',
                    project_home_directory: '',
                    project_description: '',
                    llm_provider: '',
                    openrouter_model: '',
                    gemini_model: '',

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
                detail: 'Project configuration saved successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error saving project configuration:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to save project configuration',
                life: 3000
            });
        } finally {
            setSaving(false);
        }
    };

    const testGitConnection = async () => {
        if (!projectConfig.git_repository_url) {
            toastRef.current.show({
                severity: 'warn',
                summary: 'Warning',
                detail: 'Please enter a Git repository URL',
                life: 3000
            });
            return;
        }

        try {
            setTestingConnection(true);
            const response = await fetch('/api/project/test-git-connection', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({url: projectConfig.git_repository_url}),
            });

            if (!response.ok) {
                throw new Error('Failed to connect to Git repository');
            }

            const data = await response.json();

            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: data.message || 'Successfully connected to Git repository',
                life: 3000
            });
        } catch (error) {
            console.error('Error testing Git connection:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to connect to Git repository',
                life: 3000
            });
        } finally {
            setTestingConnection(false);
        }
    };

    const handleInputChange = (field, value) => {
        setProjectConfig({
            ...projectConfig,
            [field]: value
        });
    };

    if (loading) {
        return <div>Loading project configuration...</div>;
    }

    return (
        <div className="project-container">
            <Toast ref={toastRef}/>

            <div className="flex justify-content-between align-items-center mb-3">
                <h2>Project Configuration</h2>
                <div className="flex gap-2">
                    <Button
                        label="Test Git Connection"
                        icon="pi pi-github"
                        className="p-button-info"
                        onClick={testGitConnection}
                        loading={testingConnection}
                        disabled={!projectConfig.git_repository_url || saving}
                    />
                    <Button
                        label="Save"
                        icon="pi pi-save"
                        className="p-button-success"
                        onClick={saveProjectConfig}
                        loading={saving}
                    />
                </div>
            </div>

            <Card className="project-card">
                <div className="p-fluid">
                    <div className="field">
                        <label htmlFor="git_repository_url">Git Repository URL</label>
                        <InputText
                            id="git_repository_url"
                            value={projectConfig.git_repository_url}
                            onChange={(e) => handleInputChange('git_repository_url', e.target.value)}
                            placeholder="https://github.com/username/repository.git"
                            className="w-full"
                        />
                        <small className="text-muted">The URL of the Git repository associated with this
                            project.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="project_home_directory">Project Home Directory</label>
                        <InputText
                            id="project_home_directory"
                            value={projectConfig.project_home_directory}
                            onChange={(e) => handleInputChange('project_home_directory', e.target.value)}
                            placeholder="/path/to/project"
                            className="w-full"
                        />
                        <small className="text-muted">The root directory for the project on the local file
                            system.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="project_description">Project Description</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.project_description}
                                onChange={(value) => handleInputChange('project_description', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">A descriptive summary of the project. Markdown is
                            supported.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="llm_provider">LLM Provider</label>
                        <Dropdown
                            id="llm_provider"
                            value={projectConfig.llm_provider}
                            options={llmProviderOptions}
                            onChange={(e) => handleInputChange('llm_provider', e.value)}
                            placeholder="Select an LLM Provider"
                            className="w-full"
                        />
                        <small className="text-muted">The LLM provider to use for AI-powered features. If not selected, OpenRouter will be used by default.</small>
                    </div>

                    {projectConfig.llm_provider === 'OpenRouter' && (
                        <div className="field">
                            <label htmlFor="openrouter_model">OpenRouter Model</label>
                            <Dropdown
                                id="openrouter_model"
                                value={projectConfig.openrouter_model}
                                options={openrouterModelOptions}
                                onChange={(e) => handleInputChange('openrouter_model', e.value)}
                                placeholder="Select an OpenRouter Model"
                                className="w-full"
                            />
                            <small className="text-muted">The model to use with OpenRouter. If not selected, the default model will be used.</small>
                        </div>
                    )}

                    {projectConfig.llm_provider === 'Gemini' && (
                        <div className="field">
                            <label htmlFor="gemini_model">Gemini Model</label>
                            <Dropdown
                                id="gemini_model"
                                value={projectConfig.gemini_model}
                                options={geminiModelOptions}
                                onChange={(e) => handleInputChange('gemini_model', e.value)}
                                placeholder="Select a Gemini Model"
                                className="w-full"
                            />
                            <small className="text-muted">The model to use with Gemini. If not selected, the default model will be used.</small>
                        </div>
                    )}

                    <h3>LLM Prompts Configuration</h3>
                    <p className="text-muted">Configure the prompts used by the LLM for various features. Leave empty to use default prompts.</p>

                    <div className="field">
                        <label htmlFor="auto_complete_system_prompt">Auto Complete System Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.auto_complete_system_prompt}
                                onChange={(value) => handleInputChange('auto_complete_system_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">System prompt for auto-completing block descriptions.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="auto_complete_user_prompt">Auto Complete User Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.auto_complete_user_prompt}
                                onChange={(value) => handleInputChange('auto_complete_user_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">User prompt template for auto-completing block descriptions. Use {} as a placeholder for the description.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="enhance_description_system_prompt">Enhance Description System Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.enhance_description_system_prompt}
                                onChange={(value) => handleInputChange('enhance_description_system_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">System prompt for enhancing block descriptions.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="enhance_description_user_prompt">Enhance Description User Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.enhance_description_user_prompt}
                                onChange={(value) => handleInputChange('enhance_description_user_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">User prompt template for enhancing block descriptions. Use {} as a placeholder for the description.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="generate_tasks_system_prompt">Generate Tasks System Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.generate_tasks_system_prompt}
                                onChange={(value) => handleInputChange('generate_tasks_system_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">System prompt for generating tasks from block descriptions.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="generate_tasks_user_prompt">Generate Tasks User Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.generate_tasks_user_prompt}
                                onChange={(value) => handleInputChange('generate_tasks_user_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">User prompt template for generating tasks from block descriptions. Use {} as a placeholder for the description.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="process_markdown_spec_system_prompt">Process Markdown Spec System Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.process_markdown_spec_system_prompt}
                                onChange={(value) => handleInputChange('process_markdown_spec_system_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">System prompt for processing markdown specifications.</small>
                    </div>

                    <div className="field">
                        <label htmlFor="process_markdown_spec_user_prompt">Process Markdown Spec User Prompt</label>
                        <div className="monaco-editor-container">
                            <Editor
                                height="100px"
                                defaultLanguage="markdown"
                                theme="vs-dark"
                                value={projectConfig.process_markdown_spec_user_prompt}
                                onChange={(value) => handleInputChange('process_markdown_spec_user_prompt', value || '')}
                                options={{
                                    minimap: {enabled: false},
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    lineNumbers: 'on',
                                    automaticLayout: true
                                }}
                            />
                        </div>
                        <small className="text-muted">User prompt template for processing markdown specifications. Use {} as a placeholder for the markdown content.</small>
                    </div>
                </div>
            </Card>
        </div>
    );
};

export default ProjectView;
