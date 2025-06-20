import React, { useState, useEffect } from 'react';
import { Dialog } from 'primereact/dialog';
import { Button } from 'primereact/button';
import { InputText } from 'primereact/inputtext';
import { InputTextarea } from 'primereact/inputtextarea';
import { Dropdown } from 'primereact/dropdown';
import { Chip } from 'primereact/chip';
import { classNames } from 'primereact/utils';
import './TaskDialog.css';

/**
 * TaskDialog component for creating and editing tasks
 * 
 * @param {Object} props - Component props
 * @param {boolean} props.visible - Whether the dialog is visible
 * @param {Function} props.onHide - Function to call when the dialog is hidden
 * @param {Object} props.task - Task object to edit (null for new task)
 * @param {string} props.blockId - ID of the block the task belongs to
 * @param {Function} props.onSave - Function to call when the task is saved
 */
const TaskDialog = ({ visible, onHide, task, blockId, onSave }) => {
    // Define the initial task state
    const initialTask = {
        task_id: '',
        task_name: '',
        description: '',
        acceptance_criteria: [],
        dependencies: [],
        estimated_effort: '',
        files_affected: [],
        function_signatures: [],
        testing_requirements: [],
        log: '',
        commit_id: '',
        status: 'TODO'
    };

    // State for the task form
    const [taskData, setTaskData] = useState(initialTask);
    
    // State for form validation
    const [submitted, setSubmitted] = useState(false);
    
    // State for array input fields
    const [newCriterion, setNewCriterion] = useState('');
    const [newDependency, setNewDependency] = useState('');
    const [newFile, setNewFile] = useState('');
    const [newFunction, setNewFunction] = useState('');
    const [newTestReq, setNewTestReq] = useState('');

    // Effort options
    const effortOptions = [
        { label: 'Small', value: 'S' },
        { label: 'Medium', value: 'M' },
        { label: 'Large', value: 'L' }
    ];

    // Initialize the form when the dialog is opened or the task changes
    useEffect(() => {
        if (visible) {
            if (task) {
                // Editing existing task
                setTaskData(task);
            } else {
                // Creating new task
                setTaskData({
                    ...initialTask,
                    task_id: generateTaskId()
                });
            }
            setSubmitted(false);
        }
    }, [visible, task]);

    // Generate a random task ID
    const generateTaskId = () => {
        return Math.random().toString(36).substring(2, 8);
    };

    // Handle input changes
    const handleInputChange = (e) => {
        const { name, value } = e.target;
        setTaskData(prev => ({
            ...prev,
            [name]: value
        }));
    };

    // Handle array field additions
    const handleAddArrayItem = (field, value, setValue) => {
        if (value.trim()) {
            setTaskData(prev => ({
                ...prev,
                [field]: [...prev[field], value.trim()]
            }));
            setValue('');
        }
    };

    // Handle array field removals
    const handleRemoveArrayItem = (field, index) => {
        setTaskData(prev => ({
            ...prev,
            [field]: prev[field].filter((_, i) => i !== index)
        }));
    };

    // Handle form submission
    const handleSubmit = () => {
        setSubmitted(true);

        // Validate required fields
        if (taskData.task_id && taskData.task_name) {
            onSave(blockId, taskData);
            onHide();
        }
    };

    // Render array field items
    const renderArrayItems = (items, field) => {
        return (
            <div className="flex flex-wrap gap-2 mt-2">
                {items.map((item, index) => (
                    <Chip
                        key={index}
                        label={item}
                        removable
                        onRemove={() => handleRemoveArrayItem(field, index)}
                    />
                ))}
            </div>
        );
    };

    // Dialog footer with cancel and save buttons
    const dialogFooter = (
        <div>
            <Button
                label="Cancel"
                icon="pi pi-times"
                className="p-button-text"
                onClick={onHide}
            />
            <Button
                label="Save"
                icon="pi pi-check"
                className="p-button-success"
                onClick={handleSubmit}
            />
        </div>
    );

    return (
        <Dialog
            header={task ? "Edit Task" : "Create Task"}
            visible={visible}
            style={{ width: '50vw' }}
            onHide={onHide}
            footer={dialogFooter}
            modal
            className="task-dialog"
        >
            <div className="p-fluid">
                {/* Task ID - Read-only when editing */}
                <div className="field">
                    <label htmlFor="task_id">Task ID</label>
                    <InputText
                        id="task_id"
                        name="task_id"
                        value={taskData.task_id}
                        readOnly={!!task}
                        disabled={!!task}
                        className={classNames({'p-invalid': submitted && !taskData.task_id})}
                    />
                    {submitted && !taskData.task_id && <small className="p-error">Task ID is required.</small>}
                </div>

                {/* Task Name */}
                <div className="field">
                    <label htmlFor="task_name">Task Name</label>
                    <InputText
                        id="task_name"
                        name="task_name"
                        value={taskData.task_name}
                        onChange={handleInputChange}
                        className={classNames({ 'p-invalid': submitted && !taskData.task_name })}
                    />
                    {submitted && !taskData.task_name && <small className="p-error">Task name is required.</small>}
                </div>

                {/* Description */}
                <div className="field">
                    <label htmlFor="description">Description</label>
                    <InputTextarea
                        id="description"
                        name="description"
                        value={taskData.description}
                        onChange={handleInputChange}
                        rows={3}
                    />
                </div>

                {/* Acceptance Criteria */}
                <div className="field">
                    <label>Acceptance Criteria</label>
                    <div className="flex gap-2">
                        <InputText
                            value={newCriterion}
                            onChange={(e) => setNewCriterion(e.target.value)}
                            placeholder="Add criterion"
                            className="w-full"
                        />
                        <Button
                            icon="pi pi-plus"
                            className="p-button-success"
                            onClick={() => handleAddArrayItem('acceptance_criteria', newCriterion, setNewCriterion)}
                            disabled={!newCriterion.trim()}
                        />
                    </div>
                    {renderArrayItems(taskData.acceptance_criteria, 'acceptance_criteria')}
                </div>

                {/* Dependencies */}
                <div className="field">
                    <label>Dependencies</label>
                    <div className="flex gap-2">
                        <InputText
                            value={newDependency}
                            onChange={(e) => setNewDependency(e.target.value)}
                            placeholder="Add dependency"
                            className="w-full"
                        />
                        <Button
                            icon="pi pi-plus"
                            className="p-button-success"
                            onClick={() => handleAddArrayItem('dependencies', newDependency, setNewDependency)}
                            disabled={!newDependency.trim()}
                        />
                    </div>
                    {renderArrayItems(taskData.dependencies, 'dependencies')}
                </div>

                {/* Estimated Effort */}
                <div className="field">
                    <label htmlFor="estimated_effort">Estimated Effort</label>
                    <Dropdown
                        id="estimated_effort"
                        name="estimated_effort"
                        value={taskData.estimated_effort}
                        options={effortOptions}
                        onChange={handleInputChange}
                        placeholder="Select effort level"
                    />
                </div>

                {/* Files Affected */}
                <div className="field">
                    <label>Files Affected</label>
                    <div className="flex gap-2">
                        <InputText
                            value={newFile}
                            onChange={(e) => setNewFile(e.target.value)}
                            placeholder="Add file"
                            className="w-full"
                        />
                        <Button
                            icon="pi pi-plus"
                            className="p-button-success"
                            onClick={() => handleAddArrayItem('files_affected', newFile, setNewFile)}
                            disabled={!newFile.trim()}
                        />
                    </div>
                    {renderArrayItems(taskData.files_affected, 'files_affected')}
                </div>

                {/* Function Signatures */}
                <div className="field">
                    <label>Function Signatures</label>
                    <div className="flex gap-2">
                        <InputText
                            value={newFunction}
                            onChange={(e) => setNewFunction(e.target.value)}
                            placeholder="Add function signature"
                            className="w-full"
                        />
                        <Button
                            icon="pi pi-plus"
                            className="p-button-success"
                            onClick={() => handleAddArrayItem('function_signatures', newFunction, setNewFunction)}
                            disabled={!newFunction.trim()}
                        />
                    </div>
                    {renderArrayItems(taskData.function_signatures, 'function_signatures')}
                </div>

                {/* Testing Requirements */}
                <div className="field">
                    <label>Testing Requirements</label>
                    <div className="flex gap-2">
                        <InputText
                            value={newTestReq}
                            onChange={(e) => setNewTestReq(e.target.value)}
                            placeholder="Add testing requirement"
                            className="w-full"
                        />
                        <Button
                            icon="pi pi-plus"
                            className="p-button-success"
                            onClick={() => handleAddArrayItem('testing_requirements', newTestReq, setNewTestReq)}
                            disabled={!newTestReq.trim()}
                        />
                    </div>
                    {renderArrayItems(taskData.testing_requirements, 'testing_requirements')}
                </div>

                {/* Read-only fields when editing */}
                {task && (
                    <>
                        {/* Log */}
                        <div className="field">
                            <label htmlFor="log">Log</label>
                            <InputTextarea
                                id="log"
                                value={taskData.log}
                                readOnly
                                rows={2}
                            />
                        </div>

                        {/* Commit ID */}
                        <div className="field">
                            <label htmlFor="commit_id">Commit ID</label>
                            <InputText
                                id="commit_id"
                                value={taskData.commit_id}
                                readOnly
                            />
                        </div>

                        {/* Status */}
                        <div className="field">
                            <label htmlFor="status">Status</label>
                            <InputText
                                id="status"
                                value={taskData.status}
                                readOnly
                            />
                        </div>
                    </>
                )}
            </div>
        </Dialog>
    );
};

export default TaskDialog;