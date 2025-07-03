import React, {useEffect, useRef, useState} from 'react';
import {Card} from 'primereact/card';
import {Button} from 'primereact/button';
import {Toast} from 'primereact/toast';
import {TabMenu} from 'primereact/tabmenu';
import {Editor} from '@monaco-editor/react';
import {Tooltip} from 'primereact/tooltip';
import {Dropdown} from 'primereact/dropdown';
import './ProjectView.css';

const PromptSettingsView = ({setActiveView}) => {
    const [projectConfig, setProjectConfig] = useState({
        // User-configurable prompts
        selected_profession_id: '',
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
    const [activePromptIndex, setActivePromptIndex] = useState(0);
    const [professions, setProfessions] = useState([]);
    const [professionCategories, setProfessionCategories] = useState([]);
    const [loadingProfessions, setLoadingProfessions] = useState(false);

    // Create a ref for the toast
    const toastRef = useRef(null);

    // Define the prompt types for the tabs
    const promptTabs = [
        {label: 'Auto Complete', icon: 'pi pi-pencil'},
        {label: 'Enhance Description', icon: 'pi pi-file-edit'},
        {label: 'Generate Tasks', icon: 'pi pi-list'},
        {label: 'Process Specification', icon: 'pi pi-file'}
    ];

    useEffect(() => {
        fetchProjectConfig();
        fetchProfessions();
    }, []);

    const fetchProfessions = async () => {
        try {
            setLoadingProfessions(true);
            const response = await fetch('/api/project/professions');

            if (!response.ok) {
                throw new Error('Failed to fetch professions');
            }

            const data = await response.json();

            // Process the professions data
            const allProfessions = [];
            const categories = [];

            data.categories.forEach(category => {
                categories.push({
                    name: category.name,
                    code: category.name
                });

                category.professions.forEach(profession => {
                    allProfessions.push({
                        id: profession.id,
                        name: profession.name,
                        category: category.name
                    });
                });
            });

            setProfessions(allProfessions);
            setProfessionCategories(categories);
        } catch (error) {
            console.error('Error fetching professions:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to load professions',
                life: 3000
            });
        } finally {
            setLoadingProfessions(false);
        }
    };

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

    const handleProfessionChange = async (e) => {
        const selectedProfessionId = e.value;

        // Update the selected profession in the project config
        setProjectConfig({
            ...projectConfig,
            selected_profession_id: selectedProfessionId
        });

        try {
            // Make a request to get the profession-specific prompts
            const response = await fetch(`/api/project/professions/${selectedProfessionId}/prompts`);

            if (!response.ok) {
                // If the endpoint doesn't exist yet, we'll just save the selected profession
                // and use the existing prompts
                console.warn('Profession-specific prompts endpoint not available');
                return;
            }

            const data = await response.json();

            // Update the prompts with the profession-specific ones
            setProjectConfig(prevConfig => ({
                ...prevConfig,
                selected_profession_id: selectedProfessionId,
                auto_complete_system_prompt: data.auto_complete_system_prompt,
                auto_complete_user_prompt: data.auto_complete_user_prompt,
                enhance_description_system_prompt: data.enhance_description_system_prompt,
                enhance_description_user_prompt: data.enhance_description_user_prompt,
                generate_tasks_system_prompt: data.generate_tasks_system_prompt,
                generate_tasks_user_prompt: data.generate_tasks_user_prompt,
                generate_tasks_system_prompt_mcp: data.generate_tasks_system_prompt_mcp,
                generate_tasks_user_prompt_mcp: data.generate_tasks_user_prompt_mcp,
                process_specification_system_prompt: data.process_specification_system_prompt,
                process_specification_user_prompt: data.process_specification_user_prompt,
                process_specification_system_prompt_mcp: data.process_specification_system_prompt_mcp,
                process_specification_user_prompt_mcp: data.process_specification_user_prompt_mcp
            }));

            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: `Loaded prompts for ${professions.find(p => p.id === selectedProfessionId)?.name}`,
                life: 3000
            });
        } catch (error) {
            console.error('Error loading profession-specific prompts:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to load profession-specific prompts',
                life: 3000
            });
        }
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
                                <i className="pi pi-question-circle ml-2 auto-complete-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
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
                        </div>
                        <div className="field">
                            <label htmlFor="auto_complete_user_prompt">
                                User Prompt
                                <Tooltip target=".auto-complete-user-help" position="right">
                                    User prompt template for auto-completing block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 auto-complete-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
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
                                <i className="pi pi-question-circle ml-2 enhance-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
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
                        </div>
                        <div className="field">
                            <label htmlFor="enhance_description_user_prompt">
                                User Prompt
                                <Tooltip target=".enhance-user-help" position="right">
                                    User prompt template for enhancing block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 enhance-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
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
                                <i className="pi pi-question-circle ml-2 tasks-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
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
                        </div>
                        <div className="field">
                            <label htmlFor="generate_tasks_user_prompt">
                                User Prompt
                                <Tooltip target=".tasks-user-help" position="right">
                                    User prompt template for generating tasks from block descriptions. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 tasks-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
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
                        </div>
                        <div className="field">
                            <label htmlFor="generate_tasks_system_prompt_mcp">
                                System Prompt ==MCP==
                                <Tooltip target=".tasks-system-help" position="right">
                                    System prompt for generating tasks from block descriptions ==using MCP Tools==.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 tasks-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.generate_tasks_system_prompt_mcp}
                                    onChange={(value) => handleInputChange('generate_tasks_system_prompt_mcp', value || '')}
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
                            <label htmlFor="generate_tasks_user_prompt_mcp">
                                User Prompt ==MCP==
                                <Tooltip target=".tasks-user-help" position="right">
                                    User prompt template for generating tasks from block descriptions ==using MCP Tools==. Use {} as a placeholder for the description.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 tasks-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.generate_tasks_user_prompt_mcp}
                                    onChange={(value) => handleInputChange('generate_tasks_user_prompt_mcp', value || '')}
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
                    </>
                );
            case 3: // Process Specification
                return (
                    <>
                        <div className="field">
                            <label htmlFor="process_specification_system_prompt">
                                System Prompt
                                <Tooltip target=".markdown-system-help" position="right">
                                    System prompt for processing markdown specifications.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_specification_system_prompt}
                                    onChange={(value) => handleInputChange('process_specification_system_prompt', value || '')}
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
                            <label htmlFor="process_specification_user_prompt">
                                User Prompt
                                <Tooltip target=".markdown-user-help" position="right">
                                    User prompt template for processing markdown specifications. Use {} as a placeholder for the markdown content.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_specification_user_prompt}
                                    onChange={(value) => handleInputChange('process_specification_user_prompt', value || '')}
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
                            <label htmlFor="process_specification_system_prompt_mcp">
                                System Prompt ==MCP==
                                <Tooltip target=".markdown-system-help" position="right">
                                    System prompt for processing markdown specifications ==using MCP Tools==.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-system-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="100px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_specification_system_prompt_mcp}
                                    onChange={(value) => handleInputChange('process_specification_system_prompt_mcp', value || '')}
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
                            <label htmlFor="process_specification_user_prompt_mcp">
                                User Prompt ==MCP==
                                <Tooltip target=".markdown-user-help" position="right">
                                    User prompt template for processing markdown specifications ==using MCP Tools==. Use {} as a placeholder for the markdown content.
                                </Tooltip>
                                <i className="pi pi-question-circle ml-2 markdown-user-help" style={{cursor: 'pointer'}}></i>
                            </label>
                            <div className="monaco-editor-container">
                                <Editor
                                    height="300px"
                                    defaultLanguage="markdown"
                                    theme="vs-dark"
                                    value={projectConfig.process_specification_user_prompt_mcp}
                                    onChange={(value) => handleInputChange('process_specification_user_prompt_mcp', value || '')}
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
            <Toast ref={toastRef}/>

            <div className="flex justify-content-between align-items-center mb-3">
                <div className="flex align-items-center">
                    <Button
                        icon="pi pi-arrow-left"
                        className="p-button-text mr-2"
                        onClick={() => setActiveView('project')}
                        tooltip="Back to Project Settings"
                        tooltipOptions={{position: 'bottom'}}
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

            <div className="flex flex-column mb-3">
                <div className="p-field mb-3">
                    <label htmlFor="profession" className="font-bold mb-2 block">Profession</label>
                    <Dropdown
                        id="profession"
                        value={projectConfig.selected_profession_id}
                        options={professions}
                        onChange={handleProfessionChange}
                        optionLabel="name"
                        optionValue="id"
                        placeholder="Select a profession"
                        className="w-full"
                        filter
                        showClear
                        loading={loadingProfessions}
                    />
                    <small className="text-muted">Select a profession to load profession-specific prompts</small>
                </div>

                <TabMenu
                    model={promptTabs}
                    activeIndex={activePromptIndex}
                    onTabChange={(e) => setActivePromptIndex(e.index)}
                />
            </div>

            <Card className="project-card">
                <div className="p-fluid">
                    {renderPromptEditor()}
                </div>
            </Card>
        </div>
    );
};

export default PromptSettingsView;
