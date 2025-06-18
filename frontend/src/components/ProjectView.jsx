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
        gemini_model: ''
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
        { label: 'Gemini 2.5 Flash', value: 'google/gemini-2.5-flash-preview-05-20' },
        { label: 'Gemini 1.5 Pro', value: 'google/gemini-1.5-pro-latest' },
        { label: 'Claude 3 Opus', value: 'anthropic/claude-3-opus:beta' },
        { label: 'Claude 3 Sonnet', value: 'anthropic/claude-3-sonnet:beta' },
        { label: 'GPT-4o', value: 'openai/gpt-4o' }
    ];

    // Gemini model options
    const geminiModelOptions = [
        { label: 'Gemini 2.5 Flash', value: 'gemini-2.5-flash-preview-05-20' },
        { label: 'Gemini 1.5 Pro', value: 'gemini-1.5-pro-latest' },
        { label: 'Gemini Pro', value: 'gemini-pro' }
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
                    gemini_model: ''
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
                                height="200px"
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
                </div>
            </Card>
        </div>
    );
};

export default ProjectView;
