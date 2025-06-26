import React, {useState, useEffect, useRef} from 'react';
import {Card} from 'primereact/card';
import {InputText} from 'primereact/inputtext';
import {Button} from 'primereact/button';
import {Toast} from 'primereact/toast';
import {Editor} from '@monaco-editor/react';
import {Dropdown} from 'primereact/dropdown';
import {TabMenu} from 'primereact/tabmenu';
import {Tooltip} from 'primereact/tooltip';
import './ProjectView.css';

const ProjectView = ({ setActiveView }) => {
    const [activeTabIndex, setActiveTabIndex] = useState(0);

    // Define the tabs for the project configuration
    const projectTabs = [
        { label: 'Project Settings', icon: 'pi pi-home' },
        { label: 'LLM Settings', icon: 'pi pi-cog' },
        { label: 'Prompt Settings', icon: 'pi pi-pencil', command: () => setActiveView('promptSettings') }
    ];
    const [projectConfig, setProjectConfig] = useState({
        git_repository_url: '',
        project_home_directory: '',
        project_description: '',
        main_branch: '',
        llm_provider: '',
        openrouter_model: '',
        gemini_model: '',
        anthropic_model: '',

        // User-configurable prompts
        auto_complete_system_prompt: '',
        auto_complete_user_prompt: '',
        enhance_description_system_prompt: '',
        enhance_description_user_prompt: '',
        generate_tasks_system_prompt: '',
        generate_tasks_user_prompt: '',
        generate_tasks_system_prompt_mcp: '',
        generate_tasks_user_prompt_mcp: '',
        process_specification_system_prompt: '',
        process_specification_user_prompt: '',
        process_specification_system_prompt_mcp: '',
        process_specification_user_prompt_mcp: ''
    });

    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [testingConnection, setTestingConnection] = useState(false);
    const [branches, setBranches] = useState([]);
    const [loadingBranches, setLoadingBranches] = useState(false);

    // LLM provider options
    const llmProviderOptions = [
        { label: 'ClaudeCode', value: 'ClaudeCode' },
        { label: 'GeminiCode', value: 'GeminiCode' },
        { label: 'OpenRouter', value: 'OpenRouter' },
        { label: 'Gemini', value: 'Gemini' },
        { label: 'Anthropic', value: 'Anthropic' }
    ];

    // OpenRouter model options
    const openrouterModelOptions = [
        { label: 'Gemini 2.5 Pro',        value: 'google/gemini-2.5-pro' },
        { label: 'Gemini 2.5 Flash',      value: 'google/gemini-2.5-flash' },
        { label: 'Claude 4 Opus',        value: 'anthropic/claude-opus-4-20250514' },
        { label: 'Claude 4 Sonnet',      value: 'anthropic/claude-sonnet-4-20250514' },
        { label: 'Claude 3.7 Sonnet',      value: 'anthropic/claude-3-7-sonnet-20250219' },
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

    // Anthropic model options
    const anthropicModelOptions = [
        {label: 'Claude Opus 4', value: 'claude-opus-4-20250514'},
        {label: 'Claude Sonnet 4', value: 'claude-sonnet-4-20250514'},
        {label: 'Claude Sonnet 3.7', value: 'claude-3-7-sonnet-20250219'}
    ];


    // Create a ref for the toast
    const toastRef = useRef(null);

    useEffect(() => {
        fetchProjectConfig().then(r => {
        });
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
                    main_branch: '',
                    llm_provider: '',
                    openrouter_model: '',
                    gemini_model: '',
                    anthropic_model: '',

                    // User-configurable prompts
                    auto_complete_system_prompt: '',
                    auto_complete_user_prompt: '',
                    enhance_description_system_prompt: '',
                    enhance_description_user_prompt: '',
                    generate_tasks_system_prompt: '',
                    generate_tasks_user_prompt: '',
                    generate_tasks_system_prompt_mcp: '',
                    generate_tasks_user_prompt_mcp: '',
                    process_specification_system_prompt: '',
                    process_specification_user_prompt: '',
                    process_specification_system_prompt_mcp: '',
                    process_specification_user_prompt_mcp: ''
                });
                setLoading(false);
                return;
            }

            if (!response.ok) {
                throw new Error('Failed to fetch project configuration');
            }

            const data = await response.json();
            setProjectConfig(data);

            // Refresh branches if project home directory is set
            if (data.project_home_directory) {
                fetchBranches().then(r => {

                });
            }

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

    const fetchBranches = async () => {

        try {
            setLoadingBranches(true);
            const response = await fetch('/api/git/branches');
            
            if (!response.ok) {
                throw new Error('Failed to fetch branches');
            }

            const data = await response.json();
            
            if (data.success) {
                setBranches(data.branches.map(branch => ({ label: branch, value: branch })));
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: `Found ${data.branches.length} branches`,
                    life: 3000
                });
            } else {
                throw new Error(data.message || 'Failed to fetch branches');
            }
        } catch (error) {
            console.error('Error fetching branches:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to fetch Git branches. Make sure the project directory contains a Git repository.',
                life: 5000
            });
            setBranches([]);
        } finally {
            setLoadingBranches(false);
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

    // Render content based on active tab
    const renderTabContent = () => {
        switch (activeTabIndex) {
            case 0: // Project Settings
                return (
                    <>
                        <div className="field">
                            <label htmlFor="git_repository_url">
                                Git Repository URL
                                <Tooltip target=".git-repo-help" position="right">
                                    The URL of the Git repository associated with this project.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 git-repo-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <InputText
                                id="git_repository_url"
                                value={projectConfig.git_repository_url}
                                onChange={(e) => handleInputChange('git_repository_url', e.target.value)}
                                placeholder="https://github.com/username/repository.git"
                                className="w-full"
                            />
                        </div>

                        <div className="field">
                            <label htmlFor="project_home_directory">
                                Project Home Directory
                                <Tooltip target=".project-dir-help" position="right">
                                    The root directory for the project on the local file system.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 project-dir-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <InputText
                                id="project_home_directory"
                                value={projectConfig.project_home_directory}
                                onChange={(e) => handleInputChange('project_home_directory', e.target.value)}
                                placeholder="/path/to/project"
                                className="w-full"
                            />
                        </div>

                        <div className="field">
                            <label htmlFor="project_description">
                                Project Description
                                <Tooltip target=".project-desc-help" position="right">
                                    A descriptive summary of the project. Markdown is supported.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 project-desc-help" style={{ cursor: 'pointer' }}></i>
                            </label>
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
                        </div>

                        <div className="field">
                            <label htmlFor="main_branch">
                                Main Branch
                                <Tooltip target=".main-branch-help" position="right">
                                    The main branch used for Git operations. Usually 'main' or 'master'.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 main-branch-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <div className="flex">
                                <Dropdown
                                    id="main_branch"
                                    value={projectConfig.main_branch}
                                    options={branches}
                                    onChange={(e) => handleInputChange('main_branch', e.value)}
                                    placeholder="Select main branch"
                                    className="flex-grow-1 mr-2"
                                    disabled={branches.length === 0}
                                />
                                <Button
                                    icon="pi pi-refresh"
                                    onClick={fetchBranches}
                                    className="p-button-info"
                                    loading={loadingBranches}
                                    tooltip="Refresh branches"
                                    disabled={!projectConfig.project_home_directory}
                                />
                            </div>
                        </div>
                    </>
                );
            case 1: // LLM Settings
                return (
                    <>
                        <div className="field">
                            <label htmlFor="llm_provider">
                                LLM Provider
                                <Tooltip target=".llm-provider-help" position="right">
                                    The LLM provider to use for AI-powered features. If not selected, ClaudeCode will be used by default.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 llm-provider-help" style={{ cursor: 'pointer' }}></i>
                            </label>
                            <Dropdown
                                id="llm_provider"
                                value={projectConfig.llm_provider}
                                options={llmProviderOptions}
                                onChange={(e) => handleInputChange('llm_provider', e.value)}
                                placeholder="Select an LLM Provider"
                                className="w-full"
                            />
                        </div>

                        {projectConfig.llm_provider === 'ClaudeCode' && (
                            <div className="field">
                                <label htmlFor="anthropic_model">
                                    Anthropic Model
                                    <Tooltip target=".anthropic-model-help" position="right">
                                        The model to use with Anthropic. If not selected, the default model will be used.
                                    </Tooltip>
                                    <i className="pi pi-question-circle ml-2 anthropic-model-help" style={{ cursor: 'pointer' }}></i>
                                </label>
                                <Dropdown
                                    id="anthropic_model"
                                    value={projectConfig.anthropic_model}
                                    options={anthropicModelOptions}
                                    onChange={(e) => handleInputChange('anthropic_model', e.value)}
                                    placeholder="Select an Anthropic Model"
                                    className="w-full"
                                />
                            </div>
                        )}

                        {projectConfig.llm_provider === 'GeminiCode' && (
                            <div className="field">
                                <label htmlFor="gemini_model">
                                    Gemini Model
                                    <Tooltip target=".gemini-model-help" position="right">
                                        The model to use with Gemini. If not selected, the default model will be used.
                                    </Tooltip>
                                    <i className="pi pi-question-circle ml-2 gemini-model-help" style={{ cursor: 'pointer' }}></i>
                                </label>
                                <Dropdown
                                    id="gemini_model"
                                    value={projectConfig.gemini_model}
                                    options={geminiModelOptions}
                                    onChange={(e) => handleInputChange('gemini_model', e.value)}
                                    placeholder="Select a Gemini Model"
                                    className="w-full"
                                />
                            </div>
                        )}

                        {projectConfig.llm_provider === 'OpenRouter' && (
                            <div className="field">
                                <label htmlFor="openrouter_model">
                                    OpenRouter Model
                                    <Tooltip target=".openrouter-model-help" position="right">
                                        The model to use with OpenRouter. If not selected, the default model will be used.
                                    </Tooltip>
                                    <i className="pi pi-question-circle ml-2 openrouter-model-help" style={{ cursor: 'pointer' }}></i>
                                </label>
                                <Dropdown
                                    id="openrouter_model"
                                    value={projectConfig.openrouter_model}
                                    options={openrouterModelOptions}
                                    onChange={(e) => handleInputChange('openrouter_model', e.value)}
                                    placeholder="Select an OpenRouter Model"
                                    className="w-full"
                                />
                            </div>
                        )}

                        {projectConfig.llm_provider === 'Gemini' && (
                            <div className="field">
                                <label htmlFor="gemini_model">
                                    Gemini Model
                                    <Tooltip target=".gemini-model-help" position="right">
                                        The model to use with Gemini. If not selected, the default model will be used.
                                    </Tooltip>
                                    <i className="pi pi-question-circle ml-2 gemini-model-help" style={{ cursor: 'pointer' }}></i>
                                </label>
                                <Dropdown
                                    id="gemini_model"
                                    value={projectConfig.gemini_model}
                                    options={geminiModelOptions}
                                    onChange={(e) => handleInputChange('gemini_model', e.value)}
                                    placeholder="Select a Gemini Model"
                                    className="w-full"
                                />
                            </div>
                        )}

                        {projectConfig.llm_provider === 'Anthropic' && (
                            <div className="field">
                                <label htmlFor="anthropic_model">
                                    Anthropic Model
                                    <Tooltip target=".anthropic-model-help" position="right">
                                        The model to use with Anthropic. If not selected, the default model will be used.
                                    </Tooltip>
                                    <i className="pi pi-question-circle ml-2 anthropic-model-help" style={{ cursor: 'pointer' }}></i>
                                </label>
                                <Dropdown
                                    id="anthropic_model"
                                    value={projectConfig.anthropic_model}
                                    options={anthropicModelOptions}
                                    onChange={(e) => handleInputChange('anthropic_model', e.value)}
                                    placeholder="Select an Anthropic Model"
                                    className="w-full"
                                />
                            </div>
                        )}
                    </>
                );
            default:
                return null;
        }
    };

    return (
        <div className="project-container">
            <Toast ref={toastRef}/>

            <div className="flex justify-content-between align-items-center mb-3">
                <h2>Project Configuration</h2>
                <div className="flex gap-2">
                    {activeTabIndex === 0 && (
                        <Button
                            label="Test Git Connection"
                            icon="pi pi-github"
                            className="p-button-info"
                            onClick={testGitConnection}
                            loading={testingConnection}
                            disabled={!projectConfig.git_repository_url || saving}
                        />
                    )}
                    <Button
                        label="Save"
                        icon="pi pi-save"
                        className="p-button-success"
                        onClick={saveProjectConfig}
                        loading={saving}
                    />
                </div>
            </div>

            <TabMenu 
                model={projectTabs} 
                activeIndex={activeTabIndex} 
                onTabChange={(e) => setActiveTabIndex(e.index)} 
                className="mb-3"
            />

            <Card className="project-card">
                <div className="p-fluid">
                    {renderTabContent()}
                </div>
            </Card>
        </div>
    );
};

export default ProjectView;
