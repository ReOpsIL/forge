import React, {useState, useEffect} from 'react';
import {Dialog} from 'primereact/dialog';
import {Button} from 'primereact/button';
import {InputText} from 'primereact/inputtext';
import {Dropdown} from 'primereact/dropdown';
import {Checkbox} from 'primereact/checkbox';
import {Card} from 'primereact/card';
import {Divider} from 'primereact/divider';
import {Message} from 'primereact/message';

const JiraConfigDialog = ({visible, onHide, onSync}) => {
    const [config, setConfig] = useState({
        jiraProject: '',
        syncMode: 'import', // 'import' or 'export'
        createBlocksFromProjects: true,
        createTasksFromIssues: true,
        includeEpics: true,
        includeStories: true,
        includeTasks: true,
        includeBugs: true,
        statusFilter: 'all', // 'all', 'open', 'closed'
        assigneeFilter: 'all' // 'all', 'me', 'unassigned'
    });

    const [availableProjects, setAvailableProjects] = useState([]);
    const [loading, setLoading] = useState(false);
    const [loadingProjects, setLoadingProjects] = useState(false);

    const syncModeOptions = [
        {label: 'Import from Jira', value: 'import'},
        {label: 'Export to Jira', value: 'export'},
        {label: 'Bidirectional Sync', value: 'bidirectional'}
    ];

    const statusFilterOptions = [
        {label: 'All Issues', value: 'all'},
        {label: 'Open Issues Only', value: 'open'},
        {label: 'Closed Issues Only', value: 'closed'}
    ];

    const assigneeFilterOptions = [
        {label: 'All Assignees', value: 'all'},
        {label: 'Assigned to Me', value: 'me'},
        {label: 'Unassigned', value: 'unassigned'}
    ];

    // Load available Jira projects when dialog opens
    useEffect(() => {
        if (visible) {
            loadJiraProjects();
        }
    }, [visible]);

    const loadJiraProjects = async () => {
        setLoadingProjects(true);
        try {
            const response = await fetch('/api/jira/projects');
            if (response.ok) {
                const data = await response.json();
                // Check if response has projects array (from JiraProjectsResponse)
                const projects = data.projects || data;
                setAvailableProjects(projects.map(p => ({
                    label: `${p.name} (${p.key})`,
                    value: p.key
                })));
            }
        } catch (error) {
            console.error('Error loading Jira projects:', error);
        } finally {
            setLoadingProjects(false);
        }
    };

    const handleSync = async () => {
        if (!config.jiraProject) {
            return;
        }

        setLoading(true);
        try {
            // Map frontend config to backend expected format
            const syncRequest = {
                jira_project: config.jiraProject,
                sync_mode: config.syncMode,
                create_blocks_from_projects: config.createBlocksFromProjects,
                create_tasks_from_issues: config.createTasksFromIssues,
                include_epics: config.includeEpics,
                include_stories: config.includeStories,
                include_tasks: config.includeTasks,
                include_bugs: config.includeBugs,
                status_filter: config.statusFilter,
                assignee_filter: config.assigneeFilter
            };
            await onSync(syncRequest);
        } catch (error) {
            console.error('Sync error:', error);
        } finally {
            setLoading(false);
        }
    };

    const handleConfigChange = (field, value) => {
        setConfig(prev => ({
            ...prev,
            [field]: value
        }));
    };

    const dialogFooter = (
        <div className="flex justify-content-between">
            <Button
                label="Cancel"
                icon="pi pi-times"
                className="p-button-text"
                onClick={onHide}
                disabled={loading}
            />
            <Button
                label="Start Sync"
                icon="pi pi-sync"
                className="p-button-primary"
                onClick={handleSync}
                disabled={loading || !config.jiraProject}
                loading={loading}
            />
        </div>
    );

    return (
        <Dialog
            header="Jira Sync Configuration"
            visible={visible}
            style={{width: '600px'}}
            onHide={onHide}
            footer={dialogFooter}
            modal
        >
            <div className="p-fluid">
                <Message
                    severity="info"
                    text="Configure how you want to sync data between Forge and Jira"
                    className="mb-4"
                />

                <Card title="Basic Configuration" className="mb-4">
                    <div className="formgrid grid">
                        <div className="field col-12">
                            <label htmlFor="jiraProject">Jira Project</label>
                            <Dropdown
                                id="jiraProject"
                                value={config.jiraProject}
                                options={availableProjects}
                                onChange={(e) => handleConfigChange('jiraProject', e.value)}
                                placeholder="Select a Jira project"
                                loading={loadingProjects}
                                filter
                                showClear
                                className="w-full"
                            />
                        </div>

                        <div className="field col-12">
                            <label htmlFor="syncMode">Sync Mode</label>
                            <Dropdown
                                id="syncMode"
                                value={config.syncMode}
                                options={syncModeOptions}
                                onChange={(e) => handleConfigChange('syncMode', e.value)}
                                placeholder="Select sync mode"
                                className="w-full"
                            />
                        </div>
                    </div>
                </Card>

                <Card title="What to Sync" className="mb-4">
                    <div className="formgrid grid">
                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="createBlocks"
                                    checked={config.createBlocksFromProjects}
                                    onChange={(e) => handleConfigChange('createBlocksFromProjects', e.checked)}
                                />
                                <label htmlFor="createBlocks">Create Blocks from Projects</label>
                            </div>
                        </div>

                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="createTasks"
                                    checked={config.createTasksFromIssues}
                                    onChange={(e) => handleConfigChange('createTasksFromIssues', e.checked)}
                                />
                                <label htmlFor="createTasks">Create Tasks from Issues</label>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card title="Issue Types" className="mb-4">
                    <div className="formgrid grid">
                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="includeEpics"
                                    checked={config.includeEpics}
                                    onChange={(e) => handleConfigChange('includeEpics', e.checked)}
                                />
                                <label htmlFor="includeEpics">Epics</label>
                            </div>
                        </div>

                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="includeStories"
                                    checked={config.includeStories}
                                    onChange={(e) => handleConfigChange('includeStories', e.checked)}
                                />
                                <label htmlFor="includeStories">Stories</label>
                            </div>
                        </div>

                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="includeTasks"
                                    checked={config.includeTasks}
                                    onChange={(e) => handleConfigChange('includeTasks', e.checked)}
                                />
                                <label htmlFor="includeTasks">Tasks</label>
                            </div>
                        </div>

                        <div className="field col-6">
                            <div className="field-checkbox">
                                <Checkbox
                                    id="includeBugs"
                                    checked={config.includeBugs}
                                    onChange={(e) => handleConfigChange('includeBugs', e.checked)}
                                />
                                <label htmlFor="includeBugs">Bugs</label>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card title="Filters">
                    <div className="formgrid grid">
                        <div className="field col-6">
                            <label htmlFor="statusFilter">Status Filter</label>
                            <Dropdown
                                id="statusFilter"
                                value={config.statusFilter}
                                options={statusFilterOptions}
                                onChange={(e) => handleConfigChange('statusFilter', e.value)}
                                className="w-full"
                            />
                        </div>

                        <div className="field col-6">
                            <label htmlFor="assigneeFilter">Assignee Filter</label>
                            <Dropdown
                                id="assigneeFilter"
                                value={config.assigneeFilter}
                                options={assigneeFilterOptions}
                                onChange={(e) => handleConfigChange('assigneeFilter', e.value)}
                                className="w-full"
                            />
                        </div>
                    </div>
                </Card>
            </div>
        </Dialog>
    );
};

export default JiraConfigDialog;