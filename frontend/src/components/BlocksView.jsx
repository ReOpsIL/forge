import React, {useState, useEffect, useRef} from 'react';
import {Card} from 'primereact/card';
import {InputTextarea} from 'primereact/inputtextarea';
import {Button} from 'primereact/button';
import {Divider} from 'primereact/divider';
import {Chip} from 'primereact/chip';
import {Panel} from 'primereact/panel';
import {InputText} from 'primereact/inputtext';
import {Checkbox} from 'primereact/checkbox';
import {ConfirmDialog, confirmDialog} from 'primereact/confirmdialog';
import {Dialog} from 'primereact/dialog';
import {Toast} from 'primereact/toast';
import Editor from '@monaco-editor/react';
import ReactMarkdown from 'react-markdown';
import './BlocksView.css';

const BlocksView = () => {
    const [blocks, setBlocks] = useState([]);
    const [loading, setLoading] = useState(true);
    const [editingDescription, setEditingDescription] = useState({});
    const [selectedTasks, setSelectedTasks] = useState({});
    const [runningTasks, setRunningTasks] = useState({});
    const [newTaskText, setNewTaskText] = useState({});
    const [editingBlockName, setEditingBlockName] = useState({});
    const [editingTask, setEditingTask] = useState({});
    const [editingTaskText, setEditingTaskText] = useState({});
    const [showNewBlockDialog, setShowNewBlockDialog] = useState(false);
    const [showMarkdownEditorDialog, setShowMarkdownEditorDialog] = useState(false);
    const [currentEditingBlock, setCurrentEditingBlock] = useState(null);
    const [newBlock, setNewBlock] = useState({
        block_id: '',
        name: '',
        description: '',
        inputs: [],
        outputs: [],
        connections: {
            input_connections: [],
            output_connections: []
        },
        todo_list: []
    });
    const [newInput, setNewInput] = useState('');
    const [newOutput, setNewOutput] = useState('');

    useEffect(() => {
        fetchBlocks();
    }, []);

    // Create a ref for the toast
    const toastRef = useRef(null);

    const fetchBlocks = async () => {
        try {
            setLoading(true);
            const response = await fetch('/api/blocks');
            if (!response.ok) {
                throw new Error('Failed to fetch blocks');
            }
            const data = await response.json();
            setBlocks(data);
        } catch (error) {
            console.error('Error fetching blocks:', error);
        } finally {
            setLoading(false);
        }
    };

    const handleDescriptionChange = (blockName, newDescription) => {
        setEditingDescription({
            ...editingDescription,
            [blockName]: newDescription
        });
    };

    const saveDescription = async (blockName) => {
        // Find the block to update
        const blockToUpdate = blocks.find(block => block.name === blockName);
        if (!blockToUpdate) return;

        // Create an updated block with the new description
        const updatedBlock = {
            ...blockToUpdate,
            description: editingDescription[blockName]
        };

        try {
            // Send the updated block to the server
            const response = await fetch('/api/blocks', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(updatedBlock),
            });

            if (!response.ok) {
                throw new Error('Failed to update block description');
            }

            // Update the blocks state
            setBlocks(blocks.map(block => {
                if (block.name === blockName) {
                    return updatedBlock;
                }
                return block;
            }));

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [blockName]: undefined
            });

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block description updated successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error updating block description:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to update block description',
                life: 3000
            });
        }
    };

    const enhanceDescription = async (blockName) => {
        // Find the block to update
        const blockToEnhance = blocks.find(block => block.name === blockName);
        if (!blockToEnhance) return;

        try {
            // Send the updated block to the server
            const response = await fetch(`/api/blocks/${blockToEnhance.name}/enhance`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(blockToEnhance),
            });

            if (!response.ok) {
                throw new Error('Failed to update block description');
            }

            // Update the blocks state
            setBlocks(blocks.map(block => {
                if (block.name === blockName) {
                    return blockToEnhance;
                }
                return block;
            }));

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [blockName]: undefined
            });

            // Show a success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block description enhanced successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error enhancing block description:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to enhance block description',
                life: 3000
            });
        }
    };

    const generateTasks = async (blockName) => {
        // Find the block to update
        const blockToEnhance = blocks.find(block => block.name === blockName);
        if (!blockToEnhance) return;

        try {
            // Send the updated block to the server
            const response = await fetch(`/api/blocks/${blockToEnhance.name}/generate-tasks`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(blockToEnhance),
            });

            if (!response.ok) {
                throw new Error('Failed to update block description');
            }

            // Update the blocks state
            setBlocks(blocks.map(block => {
                if (block.name === blockName) {
                    return blockToEnhance;
                }
                return block;
            }));

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [blockName]: undefined
            });

            // Show a success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block description enhanced successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error enhancing block description:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to enhance block description',
                life: 3000
            });
        }
    };


    // Function to start editing a block name
    const startEditingName = (block) => {
        setEditingBlockName({
            ...editingBlockName,
            [block.name]: block.name
        });
    };

    // Function to handle block name change
    const handleBlockNameChange = (oldName, newName) => {
        setEditingBlockName({
            ...editingBlockName,
            [oldName]: newName
        });
    };

    // Function to save a block name change
    const saveBlockName = async (oldName) => {
        const newName = editingBlockName[oldName];
        if (!newName || newName.trim() === '' || newName === oldName) {
            setEditingBlockName({
                ...editingBlockName,
                [oldName]: undefined
            });
            return;
        }

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.name === oldName);
        if (!blockToUpdate) return;

        // Create an updated block with the new name, preserving the block_id
        const updatedBlock = {
            ...blockToUpdate,
            name: newName
        };

        try {
            // Send the updated block to the server
            const response = await fetch('/api/blocks', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(updatedBlock),
            });

            if (!response.ok) {
                throw new Error('Failed to update block name');
            }

            // Refresh the blocks list to get updated references
            await fetchBlocks();

            // Clear the editing state
            setEditingBlockName({
                ...editingBlockName,
                [oldName]: undefined
            });

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block name updated successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error updating block name:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to update block name',
                life: 3000
            });
        }
    };

    const startEditing = (block) => {
        // Set the current editing block and show the markdown editor dialog
        setCurrentEditingBlock(block);
        setEditingDescription({
            ...editingDescription,
            [block.name]: block.description
        });
        setShowMarkdownEditorDialog(true);
    };

    // Task selection handling
    const handleTaskSelection = (blockName, taskIndex, isSelected) => {
        setSelectedTasks(prev => {
            const blockTasks = prev[blockName] || [];
            if (isSelected) {
                return {
                    ...prev,
                    [blockName]: [...blockTasks, taskIndex]
                };
            } else {
                return {
                    ...prev,
                    [blockName]: blockTasks.filter(idx => idx !== taskIndex)
                };
            }
        });
    };

    // Check if a task is selected
    const isTaskSelected = (blockName, taskIndex) => {
        return selectedTasks[blockName]?.includes(taskIndex) || false;
    };

    // Add a new task
    const addNewTask = async (blockName) => {
        if (!newTaskText[blockName] || newTaskText[blockName].trim() === '') return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.name === blockName);
        if (!blockToUpdate) return;

        try {
            const response = await fetch(`/api/blocks/${blockToUpdate.block_id}/todo`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newTaskText[blockName]),
            });

            if (!response.ok) {
                throw new Error('Failed to add task');
            }

            // Update the blocks state with the new task
            setBlocks(blocks.map(block => {
                if (block.name === blockName) {
                    return {
                        ...block,
                        todo_list: [...block.todo_list, newTaskText[blockName]]
                    };
                }
                return block;
            }));

            // Clear the new task input
            setNewTaskText({
                ...newTaskText,
                [blockName]: ''
            });
        } catch (error) {
            console.error('Error adding task:', error);
        }
    };

    // Delete selected tasks
    const deleteSelectedTasks = async (blockName) => {
        const tasksToDelete = selectedTasks[blockName] || [];
        if (tasksToDelete.length === 0) return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.name === blockName);
        if (!blockToUpdate) return;

        // Sort in descending order to avoid index shifting issues when deleting
        const sortedIndices = [...tasksToDelete].sort((a, b) => b - a);

        for (const index of sortedIndices) {
            try {
                const response = await fetch(`/api/blocks/${blockToUpdate.block_id}/todo/${index}`, {
                    method: 'DELETE',
                });

                if (!response.ok) {
                    throw new Error(`Failed to delete task at index ${index}`);
                }
            } catch (error) {
                console.error('Error deleting task:', error);
            }
        }

        // Update the blocks state by removing the deleted tasks
        setBlocks(blocks.map(block => {
            if (block.name === blockName) {
                const updatedTodoList = [...block.todo_list];
                sortedIndices.forEach(index => {
                    updatedTodoList.splice(index, 1);
                });
                return {
                    ...block,
                    todo_list: updatedTodoList
                };
            }
            return block;
        }));

        // Clear the selection for this block
        setSelectedTasks({
            ...selectedTasks,
            [blockName]: []
        });
    };

    // Start editing a task
    const startEditingTask = (blockName, taskIndex, taskText) => {
        setEditingTask({
            blockName,
            taskIndex
        });
        setEditingTaskText({
            ...editingTaskText,
            [`${blockName}-${taskIndex}`]: taskText
        });
    };

    // Handle task text change
    const handleTaskTextChange = (blockName, taskIndex, newText) => {
        setEditingTaskText({
            ...editingTaskText,
            [`${blockName}-${taskIndex}`]: newText
        });
    };

    // Save edited task
    const saveEditedTask = async (blockName, taskIndex) => {
        const newText = editingTaskText[`${blockName}-${taskIndex}`];
        if (!newText || newText.trim() === '') return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.name === blockName);
        if (!blockToUpdate) return;

        try {
            // Assuming there's a PUT endpoint for updating a task
            const response = await fetch(`/api/blocks/${blockToUpdate.block_id}/todo/${taskIndex}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newText),
            });

            if (!response.ok) {
                throw new Error('Failed to update task');
            }

            // Update the blocks state with the updated task
            setBlocks(blocks.map(block => {
                if (block.name === blockName) {
                    const updatedTodoList = [...block.todo_list];
                    updatedTodoList[taskIndex] = newText;
                    return {
                        ...block,
                        todo_list: updatedTodoList
                    };
                }
                return block;
            }));

            // Clear the editing state
            setEditingTask({});
            setEditingTaskText({
                ...editingTaskText,
                [`${blockName}-${taskIndex}`]: undefined
            });

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Task updated successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error updating task:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to update task',
                life: 3000
            });
        }
    };

    // Cancel task editing
    const cancelEditingTask = () => {
        setEditingTask({});
    };

    // Confirm deletion of multiple tasks
    const confirmDeleteTasks = (blockName) => {
        const tasksToDelete = selectedTasks[blockName] || [];
        if (tasksToDelete.length === 0) return;

        confirmDialog({
            message: `Are you sure you want to delete ${tasksToDelete.length} selected task(s)?`,
            header: 'Confirm Deletion',
            icon: 'pi pi-exclamation-triangle',
            acceptClassName: 'p-button-danger',
            accept: () => deleteSelectedTasks(blockName),
        });
    };

    // Execute a single task
    const executeTask = (blockName, taskIndex) => {
        // Simulate task execution
        setRunningTasks(prev => ({
            ...prev,
            [`${blockName}-${taskIndex}`]: true
        }));

        // Simulate task completion after a random time (1-3 seconds)
        const executionTime = Math.random() * 2000 + 1000;
        setTimeout(() => {
            setRunningTasks(prev => ({
                ...prev,
                [`${blockName}-${taskIndex}`]: false
            }));
        }, executionTime);
    };

    // Execute selected tasks or all tasks if none selected
    const executeSelectedTasks = (blockName) => {
        const tasksToExecute = selectedTasks[blockName]?.length > 0
            ? selectedTasks[blockName]
            : Array.from({length: blocks.find(b => b.name === blockName)?.todo_list.length || 0}, (_, i) => i);

        tasksToExecute.forEach(taskIndex => {
            executeTask(blockName, taskIndex);
        });
    };

    // Stop all running tasks
    const stopAllTasks = (blockName) => {
        const blockTasks = blocks.find(b => b.name === blockName)?.todo_list || [];

        blockTasks.forEach((_, index) => {
            setRunningTasks(prev => ({
                ...prev,
                [`${blockName}-${index}`]: false
            }));
        });
    };

    // Check if a task is running
    const isTaskRunning = (blockName, taskIndex) => {
        return runningTasks[`${blockName}-${taskIndex}`] || false;
    };

    // Check if any task is running for a block
    const areTasksRunning = (blockName) => {
        const blockTasks = blocks.find(b => b.name === blockName)?.todo_list || [];
        return blockTasks.some((_, index) => isTaskRunning(blockName, index));
    };

    // Function to handle adding a new input to the new block
    const handleAddInput = () => {
        if (!newInput.trim()) return;
        setNewBlock({
            ...newBlock,
            inputs: [...newBlock.inputs, newInput]
        });
        setNewInput('');
    };

    // Function to handle adding a new output to the new block
    const handleAddOutput = () => {
        if (!newOutput.trim()) return;
        setNewBlock({
            ...newBlock,
            outputs: [...newBlock.outputs, newOutput]
        });
        setNewOutput('');
    };

    // Function to handle removing an input from the new block
    const handleRemoveInput = (index) => {
        setNewBlock({
            ...newBlock,
            inputs: newBlock.inputs.filter((_, i) => i !== index)
        });
    };

    // Function to handle removing an output from the new block
    const handleRemoveOutput = (index) => {
        setNewBlock({
            ...newBlock,
            outputs: newBlock.outputs.filter((_, i) => i !== index)
        });
    };

    // Function to handle creating a new block
    const handleCreateBlock = async () => {
        if (!newBlock.name.trim() || !newBlock.description.trim()) {
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Block name and description are required',
                life: 3000
            });
            return;
        }

        try {
            const response = await fetch('/api/blocks', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newBlock),
            });

            if (!response.ok) {
                throw new Error('Failed to create block');
            }

            // Refresh the blocks list
            await fetchBlocks();

            // Reset the new block form
            setNewBlock({
                block_id: '',
                name: '',
                description: '',
                inputs: [],
                outputs: [],
                connections: {
                    input_connections: [],
                    output_connections: []
                },
                todo_list: []
            });

            // Close the dialog
            setShowNewBlockDialog(false);

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block created successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error creating block:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to create block',
                life: 3000
            });
        }
    };

    // Function to handle deleting a block
    const handleDeleteBlock = async (blockName) => {
        // Find the block to delete
        const blockToDelete = blocks.find(block => block.name === blockName);
        if (!blockToDelete) return;

        try {
            const response = await fetch(`/api/blocks/${blockToDelete.block_id}`, {
                method: 'DELETE',
            });

            if (!response.ok) {
                throw new Error('Failed to delete block');
            }

            // Refresh the blocks list
            await fetchBlocks();

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Block deleted successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error deleting block:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to delete block',
                life: 3000
            });
        }
    };

    // Function to confirm block deletion
    const confirmDeleteBlock = (blockName) => {
        confirmDialog({
            message: `Are you sure you want to delete the block "${blockName}"?`,
            header: 'Confirm Deletion',
            icon: 'pi pi-exclamation-triangle',
            acceptClassName: 'p-button-danger',
            accept: () => handleDeleteBlock(blockName),
        });
    };

    if (loading) {
        return <div>Loading blocks...</div>;
    }

    return (
        <div className="blocks-container">
            <Toast ref={toastRef}/>
            <ConfirmDialog/>

            {/* Markdown Editor Dialog */}
            <Dialog
                header="Edit Block Description"
                visible={showMarkdownEditorDialog}
                style={{width: '60vw'}}
                onHide={() => setShowMarkdownEditorDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Cancel"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => {
                                setShowMarkdownEditorDialog(false);
                                setCurrentEditingBlock(null);
                            }}
                        />
                        <Button
                            label="Save"
                            icon="pi pi-check"
                            className="p-button-success"
                            onClick={() => {
                                if (currentEditingBlock) {
                                    saveDescription(currentEditingBlock.name);
                                    setShowMarkdownEditorDialog(false);
                                    setCurrentEditingBlock(null);
                                }
                            }}
                        />
                    </div>
                }
            >
                <div className="monaco-editor-container">
                    <Editor
                        height="400px"
                        defaultLanguage="markdown"
                        theme="vs-dark"
                        value={currentEditingBlock ? editingDescription[currentEditingBlock.name] : ''}
                        onChange={(value) => {
                            if (currentEditingBlock) {
                                handleDescriptionChange(currentEditingBlock.name, value || '');
                            }
                        }}
                        options={{
                            minimap: {enabled: false},
                            scrollBeyondLastLine: false,
                            wordWrap: 'on',
                            lineNumbers: 'on',
                            automaticLayout: true
                        }}
                    />
                </div>
            </Dialog>
            <div className="flex justify-content-between align-items-center mb-3">
                <h2>Blocks</h2>
                <Button
                    label="New Block"
                    icon="pi pi-plus"
                    className="p-button-success"
                    onClick={() => setShowNewBlockDialog(true)}
                />
            </div>

            {/* New Block Dialog */}
            <Dialog
                header="Create New Block"
                visible={showNewBlockDialog}
                style={{width: '50vw'}}
                onHide={() => setShowNewBlockDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Cancel"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowNewBlockDialog(false)}
                        />
                        <Button
                            label="Create"
                            icon="pi pi-check"
                            className="p-button-success"
                            onClick={handleCreateBlock}
                            disabled={!newBlock.name.trim() || !newBlock.description.trim()}
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <label htmlFor="name">Name</label>
                        <InputText
                            id="name"
                            value={newBlock.name}
                            onChange={(e) => setNewBlock({...newBlock, name: e.target.value})}
                            required
                        />
                    </div>
                    <div className="field">
                        <label htmlFor="description">Description</label>
                        <InputTextarea
                            id="description"
                            value={newBlock.description}
                            onChange={(e) => setNewBlock({...newBlock, description: e.target.value})}
                            rows={3}
                            required
                        />
                    </div>
                    <div className="field">
                        <label>Inputs</label>
                        <div className="flex gap-2">
                            <InputText
                                value={newInput}
                                onChange={(e) => setNewInput(e.target.value)}
                                placeholder="Add input"
                                className="w-full"
                            />
                            <Button
                                icon="pi pi-plus"
                                className="p-button-success"
                                onClick={handleAddInput}
                                disabled={!newInput.trim()}
                            />
                        </div>
                        {newBlock.inputs.length > 0 && (
                            <div className="flex flex-wrap gap-2 mt-2">
                                {newBlock.inputs.map((input, index) => (
                                    <Chip
                                        key={index}
                                        label={input}
                                        removable
                                        onRemove={() => handleRemoveInput(index)}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                    <div className="field">
                        <label>Outputs</label>
                        <div className="flex gap-2">
                            <InputText
                                value={newOutput}
                                onChange={(e) => setNewOutput(e.target.value)}
                                placeholder="Add output"
                                className="w-full"
                            />
                            <Button
                                icon="pi pi-plus"
                                className="p-button-success"
                                onClick={handleAddOutput}
                                disabled={!newOutput.trim()}
                            />
                        </div>
                        {newBlock.outputs.length > 0 && (
                            <div className="flex flex-wrap gap-2 mt-2">
                                {newBlock.outputs.map((output, index) => (
                                    <Chip
                                        key={index}
                                        label={output}
                                        removable
                                        onRemove={() => handleRemoveOutput(index)}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </Dialog>

            <div className="grid">
                {blocks.map((block) => (
                    <div key={block.name} className="col-12 md:col-6 lg:col-4 p-2">
                        <Card
                            className="block-card"
                            title={
                                <div className="flex align-items-center justify-content-between">
                                    {editingBlockName[block.name] !== undefined ? (
                                        <div className="w-full">
                                            <InputText
                                                value={editingBlockName[block.name]}
                                                onChange={(e) => handleBlockNameChange(block.name, e.target.value)}
                                                className="w-full"
                                            />
                                            <div className="flex justify-content-end mt-2">
                                                <Button
                                                    icon="pi pi-check"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => saveBlockName(block.name)}
                                                    tooltip="Save name"
                                                />
                                                <Button
                                                    icon="pi pi-times"
                                                    className="p-button-sm p-button-danger ml-2"
                                                    onClick={() => setEditingBlockName({
                                                        ...editingBlockName,
                                                        [block.name]: undefined
                                                    })}
                                                    tooltip="Cancel"
                                                />
                                            </div>
                                        </div>
                                    ) : (
                                        <>
                                            <div>
                                                <span>{block.name}</span>
                                                <span className="ml-2 text-xs text-gray-500">ID: {block.block_id}</span>
                                            </div>
                                            <div className="flex">
                                                <Button
                                                    icon="pi pi-pencil"
                                                    className="p-button-sm p-button-text ml-2"
                                                    onClick={() => startEditingName(block)}
                                                    tooltip="Edit name"
                                                />
                                                <Button
                                                    icon="pi pi-trash"
                                                    className="p-button-sm p-button-text p-button-danger ml-2"
                                                    onClick={() => confirmDeleteBlock(block.name)}
                                                    tooltip="Delete block"
                                                />
                                            </div>
                                        </>
                                    )}
                                </div>
                            }
                            subTitle={
                                <div className="flex align-items-center">
                                    {editingDescription[block.name] !== undefined ? (
                                        <div className="w-full">
                                            <InputTextarea
                                                value={editingDescription[block.name]}
                                                onChange={(e) => handleDescriptionChange(block.name, e.target.value)}
                                                rows={2}
                                                className="w-full"
                                            />
                                            <div className="flex justify-content-end mt-2">
                                                <Button
                                                    icon="pi pi-check-square"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => generateTasks(block.name)}
                                                    tooltip="Generate tasks"
                                                />
                                                <Button
                                                    icon="pi pi-microchip-ai"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => enhanceDescription(block.name)}
                                                    tooltip="Enhance description"
                                                />
                                                <Button
                                                    icon="pi pi-check"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => saveDescription(block.name)}
                                                    tooltip="Save description"
                                                />
                                                <Button
                                                    icon="pi pi-times"
                                                    className="p-button-sm p-button-danger ml-2"
                                                    onClick={() => setEditingDescription({
                                                        ...editingDescription,
                                                        [block.name]: undefined
                                                    })}
                                                    tooltip="Cancel"
                                                />
                                            </div>
                                        </div>
                                    ) : (
                                        <>
                                            <div className="block-description mr-2">
                                                <ReactMarkdown>{block.description}</ReactMarkdown>
                                            </div>
                                            <Button
                                                icon="pi pi-pencil"
                                                className="p-button-sm p-button-text ml-2"
                                                onClick={() => startEditing(block)}
                                                tooltip="Edit description"
                                            />
                                        </>
                                    )}
                                </div>
                            }
                        >
                            <Divider/>

                            <Panel header="Inputs & Outputs" toggleable>
                                <div className="mb-3">
                                    <h4>Inputs:</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {block.inputs.map((input, index) => (
                                            <Chip key={index} label={input}/>
                                        ))}
                                    </div>
                                </div>

                                <div>
                                    <h4>Outputs:</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {block.outputs.map((output, index) => (
                                            <Chip key={index} label={output}/>
                                        ))}
                                    </div>
                                </div>
                            </Panel>

                            <Panel header="Connections" toggleable>
                                <div className="mb-3">
                                    <h4>Input Connections:</h4>
                                    {block.connections.input_connections.length > 0 ? (
                                        <ul className="m-0 p-0 list-none">
                                            {block.connections.input_connections.map((conn, index) => (
                                                <li key={index} className="mb-2">
                                                    <div>From: <strong>{conn.from_module}</strong></div>
                                                    <div>Type: <strong>{conn.output_type}</strong></div>
                                                    <div>ID: <strong>{conn.unique_id}</strong></div>
                                                </li>
                                            ))}
                                        </ul>
                                    ) : (
                                        <p>No input connections</p>
                                    )}
                                </div>

                                <div>
                                    <h4>Output Connections:</h4>
                                    {block.connections.output_connections.length > 0 ? (
                                        <ul className="m-0 p-0 list-none">
                                            {block.connections.output_connections.map((conn, index) => (
                                                <li key={index} className="mb-2">
                                                    <div>To: <strong>{conn.to_module}</strong></div>
                                                    <div>Type: <strong>{conn.input_type}</strong></div>
                                                    <div>ID: <strong>{conn.unique_id}</strong></div>
                                                </li>
                                            ))}
                                        </ul>
                                    ) : (
                                        <p>No output connections</p>
                                    )}
                                </div>
                            </Panel>

                            <Panel header="Task List" toggleable>
                                <div className="task-list-container">
                                    {/* Task List Controls */}
                                    <div className="task-list-controls mb-2 flex justify-content-start">
                                            <Button
                                                icon="pi pi-plus"
                                                className="p-button-sm ml-2"
                                                onClick={() => {
                                                    setNewTaskText({
                                                        ...newTaskText,
                                                        [block.name]: newTaskText[block.name] || ''
                                                    });
                                                }}
                                            />
                                            <Button
                                                icon="pi pi-play"
                                                className="p-button-sm p-button-success ml-2"
                                                onClick={() => executeSelectedTasks(block.name)}
                                                disabled={areTasksRunning(block.name)}
                                            />
                                            <Button
                                                icon="pi pi-stop"
                                                className="p-button-sm p-button-warning ml-2"
                                                onClick={() => stopAllTasks(block.name)}
                                                disabled={!areTasksRunning(block.name)}
                                            />
                                            <Button
                                                icon="pi pi-list"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    setSelectedTasks({
                                                        ...selectedTasks,
                                                        [block.name]: Array.from({length: block.todo_list.length}, (_, i) => i)
                                                    })
                                                }}
                                            />
                                            <Button
                                                icon="pi pi-times-circle"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    setSelectedTasks({
                                                        ...selectedTasks,
                                                        [block.name]: []
                                                    });
                                                }}
                                            />
                                            <Button
                                                icon="pi pi-trash"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    const tasksToDelete = selectedTasks[block.name] || [];
                                                    if (tasksToDelete.length > 0) {
                                                        confirmDeleteTasks(block.name);
                                                    }
                                                }}
                                                disabled={!selectedTasks[block.name]?.length}
                                            />

                                    </div>

                                    {/* New Task Input */}
                                    {newTaskText[block.name] !== undefined && (
                                        <div className="new-task-input mb-3 flex gap-2">
                                            <InputText
                                                value={newTaskText[block.name]}
                                                onChange={(e) => setNewTaskText({
                                                    ...newTaskText,
                                                    [block.name]: e.target.value
                                                })}
                                                placeholder="Enter new task"
                                                className="w-full"
                                            />
                                            <Button
                                                icon="pi pi-check"
                                                className="p-button-sm p-button-success ml-2"
                                                onClick={() => addNewTask(block.name)}
                                                disabled={!newTaskText[block.name]?.trim()}
                                            />
                                            <Button
                                                icon="pi pi-times"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => setNewTaskText({
                                                    ...newTaskText,
                                                    [block.name]: undefined
                                                })}
                                            />
                                        </div>
                                    )}

                                    {/* Task List */}
                                    {block.todo_list.length > 0 ? (
                                        <ul className="m-0 p-0 list-none">
                                            {block.todo_list.map((todo, index) => (
                                                <li key={index}
                                                    className="mb-2 flex align-items-center justify-content-between task-item">
                                                    <div className="flex align-items-center">
                                                        <Checkbox
                                                            checked={isTaskSelected(block.name, index)}
                                                            onChange={(e) => handleTaskSelection(block.name, index, e.checked)}
                                                            className="mr-2"
                                                            disabled={isTaskRunning(block.name, index)}
                                                        />
                                                        {editingTask.blockName === block.name && editingTask.taskIndex === index ? (
                                                            <div className="flex flex-column w-full">
                                                                <InputTextarea
                                                                    value={editingTaskText[`${block.name}-${index}`]}
                                                                    onChange={(e) => handleTaskTextChange(block.name, index, e.target.value)}
                                                                    className="task-edit-textarea"
                                                                    autoFocus
                                                                    rows={3}
                                                                    onKeyDown={(e) => {
                                                                        if (e.key === 'Enter' && e.ctrlKey) {
                                                                            saveEditedTask(block.name, index);
                                                                            e.preventDefault();
                                                                        } else if (e.key === 'Escape') {
                                                                            cancelEditingTask();
                                                                        }
                                                                    }}
                                                                />
                                                                <div className="flex justify-content-end mt-2 gap-2">
                                                                    <Button
                                                                        icon="pi pi-check"
                                                                        className="p-button-sm p-button-success ml-2"
                                                                        onClick={() => saveEditedTask(block.name, index)}
                                                                        disabled={!editingTaskText[`${block.name}-${index}`]?.trim()}
                                                                    />
                                                                    <Button
                                                                        icon="pi pi-times"
                                                                        className="p-button-sm p-button-danger ml-2"
                                                                        onClick={cancelEditingTask}
                                                                    />
                                                                </div>
                                                            </div>
                                                        ) : (
                                                            <span
                                                                className={isTaskRunning(block.name, index) ? 'task-running' : 'task-text'}
                                                                onDoubleClick={() => !isTaskRunning(block.name, index) && startEditingTask(block.name, index, todo)}
                                                            >
                                {isTaskRunning(block.name, index) && (
                                    <i className="pi pi-spin pi-spinner mr-2 text-blue-500"></i>
                                )}
                                                                {todo}
                              </span>
                                                        )}
                                                    </div>
                                                </li>
                                            ))}
                                        </ul>
                                    ) : (
                                        <p>No tasks</p>
                                    )}
                                </div>
                            </Panel>
                        </Card>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default BlocksView;
