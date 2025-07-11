import React, {useState, useEffect, useRef, useCallback} from 'react';
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
import {Accordion, AccordionTab} from 'primereact/accordion';
import Editor, { DiffEditor } from '@monaco-editor/react';
import ReactMarkdown from 'react-markdown';
import DependencyTreeView from './DependencyTreeView';
import TaskDialog from './TaskDialog';
import './BlocksView.css';

const BlocksView = ({ refreshTrigger }) => {
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
    const [showLoadingDialog, setShowLoadingDialog] = useState(false);
    const [showAutoCompleteDialog, setShowAutoCompleteDialog] = useState(false);
    const [showLogDialog, setShowLogDialog] = useState(false);
    const [showDiffDialog, setShowDiffDialog] = useState(false);
    const [showTaskDialog, setShowTaskDialog] = useState(false);
    const [currentTaskData, setCurrentTaskData] = useState(null);
    const [currentTaskBlockId, setCurrentTaskBlockId] = useState(null);
    const [currentTaskLog, setCurrentTaskLog] = useState('');
    const [currentDiff, setCurrentDiff] = useState({ original: '', modified: '' });
    const [currentCommitId, setCurrentCommitId] = useState('');
    const [isDiffLoading, setIsDiffLoading] = useState(false);
    const [filesDiff, setFilesDiff] = useState([]);
    const [selectedFile, setSelectedFile] = useState(null);
    const [autoCompleteSuggestion, setAutoCompleteSuggestion] = useState('');
    const [isAutoCompleteLoading, setIsAutoCompleteLoading] = useState(false);
    const [currentEditingBlock, setCurrentEditingBlock] = useState(null);
    const [currentImportBlock, setCurrentImportBlock] = useState(null);
    const [resolveDependencies, setResolveDependencies] = useState(false);
    const [forceCompleted, setForceCompleted] = useState(false);
    const [showDependencyTreeDialog, setShowDependencyTreeDialog] = useState(false);
    const [currentDependencyBlock, setCurrentDependencyBlock] = useState(null);
    const fileInputRef = useRef(null);
    const typingTimeoutRef = useRef(null);
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
        todo_list: {}
    });
    const [newInput, setNewInput] = useState('');
    const [newOutput, setNewOutput] = useState('');

    // Create a ref to store the current blocks data
    const blocksRef = useRef([]);

    // Update the ref whenever blocks changes
    useEffect(() => {
        blocksRef.current = blocks;
    }, [blocks]);

    // Fetch blocks when the component mounts
    useEffect(() => {
        fetchBlocks();

        // Set up an interval to check if blocks_config.json was modified
        const checkConfigInterval = setInterval(async () => {
            try {
                // Fetch blocks data to check if it has changed
                const response = await fetch('/api/blocks');
                if (!response.ok) {
                    throw new Error('Failed to fetch blocks');
                }
                const data = await response.json();

                // Compare with current blocks data using the ref
                const currentBlocks = JSON.stringify(blocksRef.current);
                const newBlocks = JSON.stringify(data);

                // If blocks data has changed, update the state
                if (currentBlocks !== newBlocks) {
                    console.log('Blocks configuration has changed, reloading...');
                    setBlocks(data);
                }
            } catch (error) {
                console.error('Error checking blocks configuration:', error);
            }
        }, 5000); // Check every 5 seconds

        // Clean up the interval when the component unmounts
        return () => clearInterval(checkConfigInterval);
    }, []); // No dependencies to avoid re-creating the interval

    // Fetch blocks when the refreshTrigger changes (i.e., when the Blocks tab is clicked)
    useEffect(() => {
        if (refreshTrigger > 0) {
            console.log('Blocks tab clicked, fetching blocks data...');
            fetchBlocks();
        }
    }, [refreshTrigger]);

    // Create a ref for the toast
    const toastRef = useRef(null);

    // Function to fetch auto-complete suggestions
    const fetchAutoCompleteSuggestion = async (block_id) => {

        // Find the block to update
        const blockToEnhance = blocks.find(block => block.block_id === block_id);
        if (!blockToEnhance) return;

        // Show loading dialog
        setShowLoadingDialog(true);

        if (!blockToEnhance.description || blockToEnhance.description.trim() === '') return;

        setIsAutoCompleteLoading(true);
        try {
            const response = await fetch('/api/blocks/auto-complete', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(blockToEnhance.description ),
            });

            if (!response.ok) {
                throw new Error('Failed to fetch auto-complete suggestion');
            }

            setShowLoadingDialog(false);

            const data = await response.json();
            setAutoCompleteSuggestion(data.suggestion);
            setShowAutoCompleteDialog(true);
        } catch (error) {
            setShowLoadingDialog(false);
            console.error('Error fetching auto-complete suggestion:', error);
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: 'Failed to fetch auto-complete suggestion',
                    life: 3000
                });
            }

        } finally {
            setIsAutoCompleteLoading(false);
            setShowLoadingDialog(false);
        }
    };

    // Function to accept the auto-complete suggestion
    const acceptAutoCompleteSuggestion = useCallback(() => {
        if (currentEditingBlock && autoCompleteSuggestion) {
            setEditingDescription({
                ...editingDescription,
                [currentEditingBlock.block_id]: autoCompleteSuggestion
            });
            setShowAutoCompleteDialog(false);
            setAutoCompleteSuggestion('');

            // Show success message
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: 'Auto-complete suggestion applied',
                    life: 3000
                });
            }
        }
    }, [currentEditingBlock, autoCompleteSuggestion, editingDescription]);

    // Function to reject the auto-complete suggestion
    const rejectAutoCompleteSuggestion = useCallback(() => {
        setShowAutoCompleteDialog(false);
        setAutoCompleteSuggestion('');
    }, []);

    const handleDescriptionChange = (block_id, newDescription) => {
        setEditingDescription({
            ...editingDescription,
            [block_id]: newDescription
        });
    };

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

    const saveDescription = async (block_id) => {
        // Find the block to update
        const blockToUpdate = blocks.find(block => block.block_id === block_id);
        if (!blockToUpdate) return;

        // Create an updated block with the new description
        const updatedBlock = {
            ...blockToUpdate,
            description: editingDescription[block_id]
        };

        // Show loading dialog
        setShowLoadingDialog(true);

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

            // Reload blocks configuration
            await fetchBlocks();

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [block_id]: undefined
            });

            // Show success message
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: 'Block description updated successfully',
                    life: 3000
                });
            }
        } catch (error) {
            console.error('Error updating block description:', error);
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: 'Failed to update block description',
                    life: 3000
                });
            }
        } finally {
            // Hide loading dialog
            setShowLoadingDialog(false);
        }
    };

    const enhanceDescription = async (block_id) => {
        // Find the block to update
        const blockToEnhance = blocks.find(block => block.block_id === block_id);
        if (!blockToEnhance) return;

        // Show loading dialog
        setShowLoadingDialog(true);

        try {
            // Send the updated block to the server
            const response = await fetch(`/api/blocks/${blockToEnhance.block_id}/enhance`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(blockToEnhance),
            });

            if (!response.ok) {
                throw new Error('Failed to update block description');
            }

            // Reload blocks configuration
            await fetchBlocks();

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [block_id]: undefined
            });

            // Show a success message
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: 'Block description enhanced successfully',
                    life: 3000
                });
            }
        } catch (error) {
            console.error('Error enhancing block description:', error);
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: 'Failed to enhance block description',
                    life: 3000
                });
            }
        } finally {
            // Hide loading dialog
            setShowLoadingDialog(false);
        }
    };

    const generateTasks = async (block_id) => {
        // Find the block to update
        const blockToEnhance = blocks.find(block => block.block_id === block_id);
        if (!blockToEnhance) return;
        // Show loading dialog
        setShowLoadingDialog(true);

        try {
            // Send the updated block to the server
            const response = await fetch(`/api/blocks/${blockToEnhance.block_id}/generate-tasks`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(blockToEnhance),
            });

            if (!response.ok) {
                throw new Error('Failed to update block description');
            }

            // Reload blocks configuration
            await fetchBlocks();

            // Clear the editing state
            setEditingDescription({
                ...editingDescription,
                [block_id]: undefined
            });

            // Show a success message
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: 'Block description enhanced successfully',
                    life: 3000
                });
            }
        } catch (error) {
            console.error('Error enhancing block description:', error);
            if (toastRef.current) {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: 'Failed to enhance block description',
                    life: 3000
                });
            }
        } finally {
            // Hide loading dialog
            setShowLoadingDialog(false);
        }
    };


    // Function to start editing a block name
    const startEditingName = (block) => {
        setEditingBlockName({
            ...editingBlockName,
            [block.block_id]: block.name
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
        const blockToUpdate = blocks.find(block => block.block_id === oldName);
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
            [block.block_id]: block.description
        });
        setShowMarkdownEditorDialog(true);
    };

    // Task selection handling
    const handleTaskSelection = (block_id, taskId, isSelected) => {
        setSelectedTasks(prev => {
            const blockTasks = prev[block_id] || [];
            if (isSelected) {
                return {
                    ...prev,
                    [block_id]: [...blockTasks, taskId]
                };
            } else {
                return {
                    ...prev,
                    [block_id]: blockTasks.filter(id => id !== taskId)
                };
            }
        });
    };

    // Check if a task is selected
    const isTaskSelected = (block_id, taskId) => {
        return selectedTasks[block_id]?.includes(taskId) || false;
    };

    // Add a new task
    const addNewTask = async (block_id) => {
        if (!newTaskText[block_id] || newTaskText[block_id].trim() === '') return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.block_id === block_id);
        if (!blockToUpdate) return;

        try {
            const response = await fetch(`/api/blocks/${blockToUpdate.block_id}/task`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ description: newTaskText[block_id] }),
            });

            if (!response.ok) {
                throw new Error('Failed to add task');
            }

            const data = await response.json();
            const newTaskId = data.task_id || Date.now().toString(); // Use server-provided ID or fallback to timestamp

            // Update the blocks state with the new task
            setBlocks(blocks.map(block => {
                if (block.block_id === block_id) {
                    return {
                        ...block,
                        todo_list: {
                            ...block.todo_list,
                            [newTaskId]: { 
                                task_id: newTaskId,
                                description: newTaskText[block_id], 
                                log: null 
                            }
                        }
                    };
                }
                return block;
            }));

            // Clear the new task input
            setNewTaskText({
                ...newTaskText,
                [block_id]: ''
            });
        } catch (error) {
            console.error('Error adding task:', error);
        }
    };

    // Delete selected tasks
    const deleteSelectedTasks = async (block_id) => {
        const taskIdsToDelete = selectedTasks[block_id] || [];
        if (taskIdsToDelete.length === 0) return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.block_id === block_id);
        if (!blockToUpdate) return;

        for (const taskId of taskIdsToDelete) {
            try {
                const response = await fetch(`/api/blocks/${blockToUpdate.block_id}/delete/${taskId}`, {
                    method: 'DELETE',
                });

                if (!response.ok) {
                    throw new Error(`Failed to delete task with ID ${taskId}`);
                }
            } catch (error) {
                console.error('Error deleting task:', error);
            }
        }

        // Update the blocks state by removing the deleted tasks
        setBlocks(blocks.map(block => {
            if (block.block_id === block_id) {
                // Create a new todo_list without the deleted tasks
                const updatedTodoList = { ...block.todo_list };
                taskIdsToDelete.forEach(taskId => {
                    delete updatedTodoList[taskId];
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
            [block_id]: []
        });
    };

    // Start editing a task
    const startEditingTask = (block_id, taskId, taskText) => {
        setEditingTask({
            block_id,
            taskId
        });
        setEditingTaskText({
            ...editingTaskText,
            [`${block_id}-${taskId}`]: taskText
        });
    };

    // Handle task text change
    const handleTaskTextChange = (block_id, taskId, newText) => {
        setEditingTaskText({
            ...editingTaskText,
            [`${block_id}-${taskId}`]: newText
        });
    };

    // Save edited task
    const saveEditedTask = async (block_id, taskId) => {
        const newText = editingTaskText[`${block_id}-${taskId}`];
        if (!newText || newText.trim() === '') return;

        // Find the block to update
        const blockToUpdate = blocks.find(block => block.block_id === block_id);
        if (!blockToUpdate) return;

        try {
            // Create a copy of the block with the updated task
            const updatedBlock = {
                ...blockToUpdate,
                todo_list: {...blockToUpdate.todo_list}
            };
            // Preserve the log while updating the description
            updatedBlock.todo_list[taskId] = {
                ...updatedBlock.todo_list[taskId],
                description: newText
            };

            // Use the update_block_handler endpoint to update the entire block
            const response = await fetch('/api/blocks', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(updatedBlock),
            });

            if (!response.ok) {
                throw new Error('Failed to update task');
            }

            // Reload blocks to ensure we have the latest data
            await fetchBlocks();

            // Clear the editing state
            setEditingTask({});
            setEditingTaskText({
                ...editingTaskText,
                [`${block_id}-${taskId}`]: undefined
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

    // Open task dialog for creating a new task
    const openCreateTaskDialog = (block_id) => {
        setCurrentTaskBlockId(block_id);
        setCurrentTaskData(null);
        setShowTaskDialog(true);
    };

    // Open task dialog for editing an existing task
    const openEditTaskDialog = (block_id, task) => {
        setCurrentTaskBlockId(block_id);
        setCurrentTaskData(task);
        setShowTaskDialog(true);
    };

    // Handle saving a task (create or update)
    const handleSaveTask = async (block_id, taskData) => {
        try {
            // Find the block to update
            const blockToUpdate = blocks.find(block => block.block_id === block_id);
            if (!blockToUpdate) return;

            // Create a copy of the block with the updated task
            const updatedBlock = {
                ...blockToUpdate,
                todo_list: {...blockToUpdate.todo_list}
            };

            // Add or update the task in the todo_list
            updatedBlock.todo_list[taskData.task_id] = taskData;

            // Send the updated block to the server
            const response = await fetch('/api/blocks', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(updatedBlock),
            });

            if (!response.ok) {
                throw new Error('Failed to save task');
            }

            // Reload blocks to ensure we have the latest data
            await fetchBlocks();

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: 'Task saved successfully',
                life: 3000
            });
        } catch (error) {
            console.error('Error saving task:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to save task',
                life: 3000
            });
        }
    };

    // Confirm deletion of multiple tasks
    const confirmDeleteTasks = (block_id) => {
        const tasksToDelete = selectedTasks[block_id] || [];
        if (tasksToDelete.length === 0) return;

        confirmDialog({
            message: `Are you sure you want to delete ${tasksToDelete.length} selected task(s)?`,
            header: 'Confirm Deletion',
            icon: 'pi pi-exclamation-triangle',
            acceptClassName: 'p-button-danger',
            accept: () => deleteSelectedTasks(block_id),
        });
    };

    // Execute a single task with Git integration
    const executeGitTask = async (block_id, taskId) => {
        // Find the block and task
        const block = blocks.find(b => b.block_id === block_id);
        if (!block) {
            throw new Error(`Block ${block_id} not found`);
        }

        const task = block.todo_list[taskId];
        if (!task) {
            throw new Error(`Task ${taskId} not found in block ${block_id}`);
        }

        // Set the task as running
        setRunningTasks(prev => ({
            ...prev,
            [`${block_id}-${task.task_id}`]: true
        }));

        try {
            // Call the API to execute the task with Git integration
            const response = await fetch('/api/git/execute-task', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    block_id: block_id,
                    task_id: task.task_id,
                    task_description: task.description,
                    resolve_dependencies: resolveDependencies,
                    force_completed: forceCompleted
                }),
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`Failed to execute task with Git: ${errorText}`);
            }

            // The task is now running in the background
            // We'll keep the running state until we refresh the blocks
            // and see that the task has been marked as completed

            // Set up a polling mechanism to check if the task has completed
            const checkTaskStatus = async () => {
                try {
                    // Fetch the latest blocks
                    const blocksResponse = await fetch('/api/blocks');
                    if (!blocksResponse.ok) {
                        throw new Error('Failed to fetch blocks');
                    }

                    const updatedBlocks = await blocksResponse.json();
                    const updatedBlock = updatedBlocks.find(b => b.block_id === block_id);

                    if (updatedBlock) {
                        // Find the task by task_id
                        const updatedTask = Object.values(updatedBlock.todo_list).find(t => t.task_id === task.task_id);

                        // Check if the task has been marked as completed
                        if (updatedTask && updatedTask.description && updatedTask.description.includes('[COMPLETED]')) {
                            // Update the blocks state
                            setBlocks(updatedBlocks);

                            // Set the task as not running
                            setRunningTasks(prev => ({
                                ...prev,
                                [`${block_id}-${task.task_id}`]: false
                            }));

                            // Show success message
                            toastRef.current.show({
                                severity: 'success',
                                summary: 'Success',
                                detail: 'Task executed successfully with Git integration',
                                life: 3000
                            });

                            return;
                        }
                    }

                    // If the task is still running, check again after 2 seconds
                    setTimeout(checkTaskStatus, 2000);
                } catch (error) {
                    console.error('Error checking task status:', error);

                    // If there's an error, stop polling and set the task as not running
                    setRunningTasks(prev => ({
                        ...prev,
                        [`${block_id}-${task.task_id}`]: false
                    }));
                }
            };

            // Start polling after 2 seconds
            setTimeout(checkTaskStatus, 2000);
        } catch (error) {
            console.error('Error executing task with Git:', error);

            // Show error message
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: `Failed to execute task with Git: ${error.message}`,
                life: 3000
            });

            // Set the task as not running
            setRunningTasks(prev => ({
                ...prev,
                [`${block_id}-${task.task_id}`]: false
            }));
        }
    };


    // Execute selected tasks with Git integration
    const executeSelectedGitTasks = (block_id) => {
        const block = blocks.find(b => b.block_id === block_id);
        if (!block) return;

        const tasksToExecute = selectedTasks[block_id]?.length > 0
            ? selectedTasks[block_id].map(index => block.todo_list[index])
            : Object.values(block.todo_list);

        tasksToExecute.forEach(task => {
            if (task) {
                executeGitTask(block_id, task.task_id);
            }
        });
    };

    // Stop all running tasks
    const stopAllTasks = (block_id) => {
        const block = blocks.find(b => b.block_id === block_id);
        if (!block) return;

        const blockTasks = Object.values(block.todo_list);

        blockTasks.forEach(task => {
            setRunningTasks(prev => ({
                ...prev,
                [`${block_id}-${task.task_id}`]: false
            }));
        });
    };

    // Check if a task is running
    const isTaskRunning = (block_id, taskId) => {
        return runningTasks[`${block_id}-${taskId}`] || false;
    };

    // Check if any task is running for a block
    const areTasksRunning = (block_id) => {
        const blockTasks = blocks.find(b => b.block_id === block_id)?.todo_list || {};
        return Object.values(blockTasks).some(task => isTaskRunning(block_id, task.task_id));
    };

    // Show task log
    const showTaskLog = (block_id, taskId) => {
        const block = blocks.find(b => b.block_id === block_id);
        if (!block) return;

        const task = block.todo_list[taskId];
        if (!task) return;

        setCurrentTaskLog(task.log || '');
        setShowLogDialog(true);
    };

    // Show task diff
    const showTaskDiff = async (block_id, taskId) => {
        const block = blocks.find(b => b.block_id === block_id);
        if (!block) return;

        const task = block.todo_list[taskId];
        if (!task) return;

        // If the task doesn't have a commit ID, show an error message
        if (!task.commit_id) {
            toastRef.current.show({
                severity: 'warn',
                summary: 'No Commit ID',
                detail: 'This task has no associated Git commit.',
                life: 3000
            });
            return;
        }

        setIsDiffLoading(true);
        setShowDiffDialog(true);

        try {
            const response = await fetch('/api/git/task-diff', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    block_id: block_id,
                    task_id: task.task_id
                }),
            });

            const data = await response.json();

            if (response.ok && data.success) {
                // Store both original and modified content for the diff editor
                setCurrentDiff({
                    original: data.original_content || '',
                    modified: data.modified_content || ''
                });
                setCurrentCommitId(data.commit_id || '');

                // Store the list of modified files
                if (data.files_diff && data.files_diff.length > 0) {
                    setFilesDiff(data.files_diff);
                    // Select the first file by default
                    setSelectedFile(data.files_diff[0]);
                } else {
                    setFilesDiff([]);
                    setSelectedFile(null);
                }
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message || 'Failed to fetch diff',
                    life: 3000
                });
                setCurrentDiff({ original: '', modified: '' });
                setCurrentCommitId('');
                setFilesDiff([]);
                setSelectedFile(null);
            }
        } catch (error) {
            console.error('Error fetching diff:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to fetch diff',
                life: 3000
            });
            setCurrentDiff({ original: '', modified: '' });
            setCurrentCommitId('');
            setFilesDiff([]);
            setSelectedFile(null);
        } finally {
            setIsDiffLoading(false);
        }
    };

    // Function to handle file selection for importing tasks
    const handleFileSelect = (block_id) => {
        setCurrentImportBlock(block_id);
        fileInputRef.current.click();
    };

    // Function to handle file upload and process markdown
    const handleFileUpload = async (event) => {
        const file = event.target.files[0];
        if (!file || !currentImportBlock) return;

        // Only accept markdown files
        if (!file.name.endsWith('.md')) {
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Please select a markdown (.md) file',
                life: 3000
            });
            return;
        }

        // Show loading dialog
        setShowLoadingDialog(true);

        try {
            // Read the file content
            const reader = new FileReader();
            const fileContent = await new Promise((resolve, reject) => {
                reader.onload = (e) => resolve(e.target.result);
                reader.onerror = (e) => reject(e);
                reader.readAsText(file);
            });

            // Send the file content to the server for processing
            const response = await fetch('/api/blocks/process-markdown', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    block_id: currentImportBlock,
                    markdown_content: fileContent
                }),
            });

            if (!response.ok) {
                throw new Error('Failed to process markdown file');
            }

            const data = await response.json();

            // Reload blocks to show the updated tasks
            await fetchBlocks();

            // Show success message
            toastRef.current.show({
                severity: 'success',
                summary: 'Success',
                detail: data.message,
                life: 3000
            });
        } catch (error) {
            console.error('Error processing markdown file:', error);
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to process markdown file',
                life: 3000
            });
        } finally {
            // Hide loading dialog
            setShowLoadingDialog(false);
            // Reset file input
            event.target.value = '';
            setCurrentImportBlock(null);
        }
    };

    // Function to convert tasks to markdown format
    const convertTasksToMarkdown = (block) => {
        let markdown = `# ${block.name} Tasks (ID: ${block.block_id})\n\n`;

        const tasks = Object.values(block.todo_list);
        if (tasks.length === 0) {
            markdown += "No tasks available.\n";
        } else {
            tasks.forEach(task => {
                markdown += `- [ ] ${task.description}\n`;
            });
        }

        return markdown;
    };

    // Function to export tasks as markdown file
    const exportTasksAsMarkdown = (block) => {
        const markdown = convertTasksToMarkdown(block);
        const filename = `${block.block_id}_${block.block_id}.md`;

        // Create a blob with the markdown content
        const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });

        // Create a URL for the blob
        const url = URL.createObjectURL(blob);

        // Create a temporary anchor element to trigger the download
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;

        // Append the anchor to the body, click it, and remove it
        document.body.appendChild(a);
        a.click();

        // Clean up
        setTimeout(() => {
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        }, 0);

        // Show success message
        toastRef.current.show({
            severity: 'success',
            summary: 'Success',
            detail: 'Tasks exported successfully',
            life: 3000
        });
    };

    // Function to handle adding a new input to the new block
    const handleAddInput = () => {
        if (!newInput.trim()) return;
        setNewBlock({
            ...newBlock,
            inputs: [...newBlock.inputs, {
                name: newInput,
                ctype: "string", // Default ctype
                description: "Input description" // Default description
            }]
        });
        setNewInput('');
    };

    // Function to handle adding a new output to the new block
    const handleAddOutput = () => {
        if (!newOutput.trim()) return;
        setNewBlock({
            ...newBlock,
            outputs: [...newBlock.outputs, {
                name: newOutput,
                ctype: "string", // Default ctype
                description: "Output description" // Default description
            }]
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
                todo_list: {}
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
    const handleDeleteBlock = async (block_id) => {
        // Find the block to delete
        const blockToDelete = blocks.find(block => block.block_id === block_id);
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
    const confirmDeleteBlock = (block_id) => {
        confirmDialog({
            message: `Are you sure you want to delete the block "${block_id}"?`,
            header: 'Confirm Deletion',
            icon: 'pi pi-exclamation-triangle',
            acceptClassName: 'p-button-danger',
            accept: () => handleDeleteBlock(block_id),
        });
    };

    if (loading) {
        return <div>Loading blocks...</div>;
    }

    return (
        <div className="blocks-container">
            <Toast ref={toastRef}/>
            <ConfirmDialog/>

            {/* Hidden file input for markdown import */}
            <input 
                type="file" 
                ref={fileInputRef} 
                style={{ display: 'none' }} 
                accept=".md" 
                onChange={handleFileUpload}
            />

            {/* Loading Dialog */}
            <Dialog
                header="Processing"
                visible={showLoadingDialog}
                style={{
                    width: '300px',
                    display: 'flex',
                    flexDirection: 'column',
                    height: '150px'
                }}
                closable={false}
                modal={true}
                showHeader={false}
            >
                <div className="flex align-items-center justify-content-center h-full">
                    <i className="pi pi-spin pi-spinner" style={{fontSize: '2rem', marginRight: '0.5rem'}}></i>
                    <span>Please wait...</span>
                </div>
            </Dialog>

            {/* Auto-Complete Dialog */}
            <Dialog
                header="Auto-Complete Suggestion"
                visible={showAutoCompleteDialog}
                style={{width: '60vw'}}
                onHide={rejectAutoCompleteSuggestion}
                footer={
                    <div>
                        <Button
                            label="Reject"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={rejectAutoCompleteSuggestion}
                            size="small"
                        />
                        <Button
                            label="Accept"
                            icon="pi pi-check"
                            className="p-button-success"
                            onClick={acceptAutoCompleteSuggestion}
                            size="small"
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <label>The AI suggests the following enhanced description:</label>
                        <div className="p-2 border-1 surface-border border-round mt-2" style={{backgroundColor: '#FFFFFF19'}}>
                            <ReactMarkdown>{autoCompleteSuggestion}</ReactMarkdown>
                        </div>
                    </div>
                </div>
            </Dialog>

            {/* Task Log Dialog */}
            <Dialog
                header="Task Execution Log"
                visible={showLogDialog}
                style={{width: '60vw'}}
                onHide={() => setShowLogDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Close"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowLogDialog(false)}
                            size="small"
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        {currentTaskLog ? (
                            <div className="p-2 border-1 surface-border border-round mt-2" 
                                 style={{
                                     backgroundColor: '#FFFFFF19', 
                                     maxHeight: '400px', 
                                     overflow: 'auto',
                                     whiteSpace: 'pre-wrap',
                                     fontFamily: 'monospace'
                                 }}>
                                {currentTaskLog}
                            </div>
                        ) : (
                            <div className="p-2 border-1 surface-border border-round mt-2 text-center">
                                No log available for this task.
                            </div>
                        )}
                    </div>
                </div>
            </Dialog>

            {/* Task Diff Dialog */}
            <Dialog
                header={`Git Diff for Commit ${currentCommitId}`}
                visible={showDiffDialog}
                style={{width: '85vw'}}
                onHide={() => setShowDiffDialog(false)}
                contentStyle={{padding: '0.5rem'}}
                footer={
                    <div>
                        <Button
                            label="Close"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowDiffDialog(false)}
                            size="small"
                        />
                    </div>
                }
            >
                <div className="p-0">
                    {isDiffLoading ? (
                        <div className="p-1 border-1 surface-border border-round text-center">
                            Loading diff...
                        </div>
                    ) : (
                        <div className="grid m-0">
                            {/* File List */}
                            <div className="col-2 p-0 pr-1">
                                <h4 className="m-0 mb-1">Modified Files</h4>
                                <div className="p-1 border-1 surface-border border-round">
                                    {filesDiff && filesDiff.length > 0 ? (
                                        <ul className="list-none p-0 m-0">
                                            {filesDiff.map((file, index) => (
                                                <li key={index} className="mb-1">
                                                    <Button
                                                        label={file.file_path}
                                                        className={`p-button-text p-button-plain w-full text-left p-1 ${selectedFile && selectedFile.file_path === file.file_path ? 'bg-primary' : ''}`}
                                                        onClick={() => setSelectedFile(file)}
                                                        size="small"
                                                    />
                                                </li>
                                            ))}
                                        </ul>
                                    ) : (
                                        <div className="text-center p-1">
                                            No modified files found.
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Diff Editor */}
                            <div className="col-10 p-0 pl-1">
                                {selectedFile ? (
                                    <div>
                                        <h4 className="m-0 mb-1">Diff for {selectedFile.file_path}</h4>
                                        <div className="monaco-editor-container">
                                            <DiffEditor
                                                height="500px"
                                                language="javascript" // Use appropriate language or detect from file extension
                                                theme="vs-dark"
                                                original={selectedFile.original_content || ''}
                                                modified={selectedFile.modified_content || ''}
                                                options={{
                                                    readOnly: true,
                                                    minimap: {enabled: false},
                                                    scrollBeyondLastLine: false,
                                                    wordWrap: 'on',
                                                    lineNumbers: 'on',
                                                    automaticLayout: true,
                                                    renderSideBySide: true
                                                }}
                                            />
                                        </div>
                                    </div>
                                ) : (currentDiff && (currentDiff.original || currentDiff.modified)) ? (
                                    <div>
                                        <h4 className="m-0 mb-1">Full Commit Diff</h4>
                                        <div className="monaco-editor-container">
                                            <DiffEditor
                                                height="500px"
                                                language="javascript" // Use appropriate language or detect from file extension
                                                theme="vs-dark"
                                                original={currentDiff.original}
                                                modified={currentDiff.modified}
                                                options={{
                                                    readOnly: true,
                                                    minimap: {enabled: false},
                                                    scrollBeyondLastLine: false,
                                                    wordWrap: 'on',
                                                    lineNumbers: 'on',
                                                    automaticLayout: true,
                                                    renderSideBySide: true
                                                }}
                                            />
                                        </div>
                                    </div>
                                ) : (
                                    <div className="p-1 border-1 surface-border border-round text-center">
                                        No diff available for this commit.
                                    </div>
                                )}
                            </div>
                        </div>
                    )}
                </div>
            </Dialog>

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
                            size="small"
                        />
                        <Button
                            label="Save"
                            icon="pi pi-check"
                            className="p-button-success"
                            onClick={() => {
                                if (currentEditingBlock) {
                                    saveDescription(currentEditingBlock.block_id);
                                    setShowMarkdownEditorDialog(false);
                                    setCurrentEditingBlock(null);
                                }
                            }}
                            size="small"
                        />
                    </div>
                }
            >
                <div className="monaco-editor-container">
                    <Editor
                        height="400px"
                        defaultLanguage="markdown"
                        theme="vs-dark"
                        value={currentEditingBlock ? editingDescription[currentEditingBlock.block_id] : ''}
                        onChange={(value) => {
                            if (currentEditingBlock) {
                                handleDescriptionChange(currentEditingBlock.block_id, value || '');
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

            {/* Dependency Tree Dialog */}
            <DependencyTreeView
                blockId={currentDependencyBlock}
                visible={showDependencyTreeDialog}
                onHide={() => setShowDependencyTreeDialog(false)}
            />

            {/* Task Dialog */}
            <TaskDialog
                visible={showTaskDialog}
                onHide={() => setShowTaskDialog(false)}
                task={currentTaskData}
                blockId={currentTaskBlockId}
                onSave={handleSaveTask}
            />

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
                            size="small"
                        />
                        <Button
                            label="Create"
                            icon="pi pi-check"
                            className="p-button-success"
                            onClick={handleCreateBlock}
                            disabled={!newBlock.name.trim() || !newBlock.description.trim()}
                            size="small"
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
                                        label={input.name}
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
                                        label={output.name}
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
                    <div key={block.block_id} className="col-12 md:col-6 lg:col-4 p-2">
                        <Card
                            className="block-card"
                            title={
                                <div className="flex align-items-center justify-content-between">
                                    {editingBlockName[block.block_id] !== undefined ? (
                                        <div className="w-full">
                                            <InputText
                                                value={editingBlockName[block.block_id]}
                                                onChange={(e) => handleBlockNameChange(block.block_id, e.target.value)}
                                                className="w-full"
                                            />
                                            <div className="flex justify-content-end mt-2">
                                                <Button
                                                    icon="pi pi-check"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => saveBlockName(block.block_id)}
                                                    tooltip="Save name"
                                                />
                                                <Button
                                                    icon="pi pi-times"
                                                    className="p-button-sm p-button-danger ml-2"
                                                    onClick={() => setEditingBlockName({
                                                        ...editingBlockName,
                                                        [block.block_id]: undefined
                                                    })}
                                                    tooltip="Cancel"
                                                />
                                            </div>
                                        </div>
                                    ) : (
                                        <>
                                            <div>
                                                <span>{block.name}</span>
                                                <span className="ml-2 text-s text-gray-500">ID: {block.block_id}</span>
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
                                                    onClick={() => confirmDeleteBlock(block.block_id)}
                                                    tooltip="Delete block"
                                                />
                                            </div>
                                        </>
                                    )}
                                </div>
                            }
                            subTitle={
                                <div className="flex align-items-center">
                                    {editingDescription[block.block_id] !== undefined ? (
                                        <div className="w-full">
                                            <InputTextarea
                                                value={editingDescription[block.block_id]}
                                                onChange={(e) => handleDescriptionChange(block.block_id, e.target.value)}
                                                rows={2}
                                                className="w-full"
                                            />
                                            <div className="flex justify-content-end mt-2">
                                                <Button
                                                    icon="pi pi-check-square"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => generateTasks(block.block_id)}
                                                    tooltip="Generate tasks"
                                                />
                                                <Button
                                                    icon="pi pi-microchip-ai"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => enhanceDescription(block.block_id)}
                                                    tooltip="Enhance description"
                                                />
                                                <Button
                                                    icon="pi pi-megaphone"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => fetchAutoCompleteSuggestion(block.block_id)}
                                                    tooltip="Auto-complete description"
                                                />
                                                <Button
                                                    icon="pi pi-check"
                                                    className="p-button-sm p-button-success ml-2"
                                                    onClick={() => saveDescription(block.block_id)}
                                                    tooltip="Save description"
                                                />
                                                <Button
                                                    icon="pi pi-times"
                                                    className="p-button-sm p-button-danger ml-2"
                                                    onClick={() => setEditingDescription({
                                                        ...editingDescription,
                                                        [block.block_id]: undefined
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
                                    <h4 style={{fontSize: '1.0rem'}}>Inputs:</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {block.inputs.map((input, index) => (
                                            <Chip key={index} label={input.name}/>
                                        ))}
                                    </div>
                                </div>

                                <div>
                                    <h4 style={{fontSize: '1.0rem'}}>Outputs:</h4>
                                    <div className="flex flex-wrap gap-2">
                                        {block.outputs.map((output, index) => (
                                            <Chip key={index} label={output.name}/>
                                        ))}
                                    </div>
                                </div>
                            </Panel>

                            <Panel header="Connections" toggleable>
                                <div className="mb-3">
                                    <h4 style={{fontSize: '1.0rem'}}>Input Connections:</h4>
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
                                        <h4 style={{fontSize: '0.7rem'}}>No input connections</h4>
                                    )}
                                </div>

                                <div>
                                    <h4 style={{fontSize: '1.0rem'}}>Output Connections:</h4>
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
                                        <h4 style={{fontSize: '0.7rem'}}>No output connections</h4>
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
                                                onClick={() => openCreateTaskDialog(block.block_id)}
                                                tooltip="Create new task"
                                                tooltipOptions={{position: 'top'}}
                                            />
                                            <div className="flex align-items-center">
                                                <Button
                                                    icon="pi pi-hammer"
                                                    className="p-button-sm p-button-info ml-2"
                                                    onClick={() => executeSelectedGitTasks(block.block_id)}
                                                    disabled={areTasksRunning(block.block_id)}
                                                    tooltip="Run Tasks"
                                                    tooltipOptions={{position: 'top'}}
                                                />
                                                <div className="ml-2 flex align-items-center">
                                                    <Checkbox
                                                        inputId="resolve-dependencies"
                                                        checked={resolveDependencies}
                                                        onChange={e => setResolveDependencies(e.checked)}
                                                        tooltip="Resolve dependency"
                                                        tooltipOptions={{position: 'top'}}
                                                    />
                                                    <label htmlFor="resolve-dependencies" className="ml-1 text-sm">Dep</label>
                                                </div>
                                                <div className="ml-2 flex align-items-center">
                                                    <Checkbox
                                                        inputId="force-completed"
                                                        checked={forceCompleted}
                                                        onChange={e => setForceCompleted(e.checked)}
                                                        tooltip="Force completed"
                                                        tooltipOptions={{position: 'top'}}
                                                    />
                                                    <label htmlFor="force-completed" className="ml-1 text-sm">Force</label>
                                                </div>
                                            </div>
                                            <Button
                                                icon="pi pi-sitemap"
                                                className="p-button-sm p-button-info ml-2"
                                                onClick={() => {
                                                    setCurrentDependencyBlock(block.block_id);
                                                    setShowDependencyTreeDialog(true);
                                                }}
                                                tooltip="View dependency tree"
                                                tooltipOptions={{position: 'top'}}
                                            />
                                            <Button
                                                icon="pi pi-exclamation-triangle"
                                                tooltip="Stop tasks execution"
                                                className="p-button-sm p-button-warning ml-2"
                                                onClick={() => stopAllTasks(block.block_id)}
                                                disabled={!areTasksRunning(block.block_id)}
                                            />
                                            <Button
                                                icon="pi pi-check-square"
                                                tooltip="Select all tasks"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    setSelectedTasks({
                                                        ...selectedTasks,
                                                        [block.block_id]: Object.keys(block.todo_list)
                                                    })
                                                }}
                                            />
                                            <Button
                                                icon="pi pi-stop"
                                                tooltip="Unselect all tasks"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    setSelectedTasks({
                                                        ...selectedTasks,
                                                        [block.block_id]: []
                                                    });
                                                }}
                                            />
                                            <Button
                                                icon="pi pi-trash"
                                                tooltip="Delete task"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => {
                                                    const tasksToDelete = selectedTasks[block.block_id] || [];
                                                    if (tasksToDelete.length > 0) {
                                                        confirmDeleteTasks(block.block_id);
                                                    }
                                                }}
                                                disabled={!selectedTasks[block.block_id]?.length}
                                            />
                                            <Button
                                                icon="pi pi-book"
                                                tooltip="Show task logs"
                                                className="p-button-sm p-button-info ml-2"
                                                onClick={() => {
                                                    const selectedTaskIndices = selectedTasks[block.block_id] || [];
                                                    if (selectedTaskIndices.length === 1) {
                                                        showTaskLog(block.block_id, selectedTaskIndices[0]);
                                                    } else {
                                                        toastRef.current.show({
                                                            severity: 'warn',
                                                            summary: 'Warning',
                                                            detail: 'Please select exactly one task to view its log',
                                                            life: 3000
                                                        });
                                                    }
                                                }}
                                                disabled={!selectedTasks[block.block_id]?.length || selectedTasks[block.block_id]?.length !== 1}
                                            />
                                            <Button
                                                icon="pi pi-code"
                                                tooltip="Show task diff"
                                                className="p-button-sm p-button-code ml-2"
                                                onClick={() => {
                                                    const selectedTaskIndices = selectedTasks[block.block_id] || [];
                                                    if (selectedTaskIndices.length === 1) {
                                                        showTaskDiff(block.block_id, selectedTaskIndices[0]);
                                                    } else {
                                                        toastRef.current.show({
                                                            severity: 'warn',
                                                            summary: 'Warning',
                                                            detail: 'Please select exactly one task to view changes',
                                                            life: 3000
                                                        });
                                                    }
                                                }}
                                                disabled={!selectedTasks[block.block_id]?.length || selectedTasks[block.block_id]?.length !== 1}
                                            />
                                            <Button
                                                icon="pi pi-file-import"
                                                className="p-button-sm p-button-help ml-2"
                                                onClick={() => handleFileSelect(block.block_id)}
                                                tooltip="Import tasks from markdown file"
                                                tooltipOptions={{position: 'top'}}
                                            />
                                            <Button
                                                icon="pi pi-file-export"
                                                className="p-button-sm p-button-help ml-2"
                                                onClick={() => exportTasksAsMarkdown(block)}
                                                tooltip="Export tasks to markdown file"
                                                tooltipOptions={{position: 'top'}}
                                            />
                                    </div>

                                    {/* New Task Input */}
                                    {newTaskText[block.block_id] !== undefined && (
                                        <div className="new-task-input mb-3 flex gap-2">
                                            <InputText
                                                value={newTaskText[block.block_id]}
                                                onChange={(e) => setNewTaskText({
                                                    ...newTaskText,
                                                    [block.block_id]: e.target.value
                                                })}
                                                placeholder="Enter new task"
                                                className="w-full"
                                            />
                                            <Button
                                                icon="pi pi-check"
                                                className="p-button-sm p-button-success ml-2"
                                                onClick={() => addNewTask(block.block_id)}
                                                disabled={!newTaskText[block.block_id]?.trim()}
                                            />
                                            <Button
                                                icon="pi pi-times"
                                                className="p-button-sm p-button-danger ml-2"
                                                onClick={() => setNewTaskText({
                                                    ...newTaskText,
                                                    [block.block_id]: undefined
                                                })}
                                            />
                                        </div>
                                    )}

                                    {/* Task List */}
                                    {Object.keys(block.todo_list).length > 0 ? (
                                        <div className="task-list-scrollable">
                                            <Accordion multiple className="w-full">
                                                {Object.values(block.todo_list).map((todo) => (
                                                    <AccordionTab 
                                                        key={todo.task_id}
                                                        headerClassName="task-accordion-header"
                                                        header={
                                                            <div className="flex align-items-center w-full">
                                                                <Checkbox
                                                                    checked={isTaskSelected(block.block_id, todo.task_id)}
                                                                    onChange={(e) => handleTaskSelection(block.block_id, todo.task_id, e.checked)}
                                                                    className="mr-2"
                                                                    disabled={isTaskRunning(block.block_id, todo.task_id)}
                                                                />
                                                                {editingTask.block_id === block.block_id && editingTask.taskId === todo.task_id ? (
                                                                    <div className="flex flex-column w-full">
                                                                        <InputTextarea
                                                                            value={editingTaskText[`${block.block_id}-${todo.task_id}`]}
                                                                            onChange={(e) => handleTaskTextChange(block.block_id, todo.task_id, e.target.value)}
                                                                            className="task-edit-textarea"
                                                                            autoFocus
                                                                            rows={3}
                                                                            onKeyDown={(e) => {
                                                                                if (e.key === 'Enter' && e.ctrlKey) {
                                                                                    saveEditedTask(block.block_id, todo.task_id);
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
                                                                                onClick={() => saveEditedTask(block.block_id, todo.task_id)}
                                                                                disabled={!editingTaskText[`${block.block_id}-${todo.task_id}`]?.trim()}
                                                                            />
                                                                            <Button
                                                                                icon="pi pi-times"
                                                                                className="p-button-sm p-button-danger ml-2"
                                                                                onClick={cancelEditingTask}
                                                                            />
                                                                        </div>
                                                                    </div>
                                                                ) : (
                                                                    <div className="flex align-items-center justify-content-between w-full">
                                                                        <span
                                                                            className={isTaskRunning(block.block_id, todo.task_id) ? 'task-running' : 'task-text'}
                                                                            title={`Task ID: ${todo.task_id}`}
                                                                        >
                                                                            {isTaskRunning(block.block_id, todo.task_id) && (
                                                                                <span className="sandclock"></span>
                                                                            )}
                                                                            <span className="task-id">[{todo.task_id}]</span> {todo.task_name || todo.description}
                                                                        </span>
                                                                        <div className="task-actions">
                                                                            <Button
                                                                                icon="pi pi-pencil"
                                                                                className="p-button-sm p-button-text"
                                                                                onClick={(e) => {
                                                                                    e.stopPropagation();
                                                                                    openEditTaskDialog(block.block_id, todo);
                                                                                }}
                                                                                tooltip="Edit task"
                                                                                tooltipOptions={{ position: 'left' }}
                                                                                disabled={isTaskRunning(block.block_id, todo.task_id)}
                                                                            />
                                                                        </div>
                                                                    </div>
                                                                )}
                                                            </div>
                                                        }
                                                    >
                                                        <div className="task-details p-3">
                                                            <div className="mb-3">
                                                                <h4 className="m-0 mb-2">Description</h4>
                                                                <p className="m-0">{todo.description}</p>
                                                            </div>

                                                            {todo.acceptance_criteria && todo.acceptance_criteria.length > 0 && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Acceptance Criteria</h4>
                                                                    <ul className="m-0 pl-3">
                                                                        {todo.acceptance_criteria.map((criteria, index) => (
                                                                            <li key={index}>{criteria}</li>
                                                                        ))}
                                                                    </ul>
                                                                </div>
                                                            )}

                                                            {todo.dependencies && todo.dependencies.length > 0 && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Dependencies</h4>
                                                                    <ul className="m-0 pl-3">
                                                                        {todo.dependencies.map((dependency, index) => (
                                                                            <li key={index}>{dependency}</li>
                                                                        ))}
                                                                    </ul>
                                                                </div>
                                                            )}

                                                            {todo.estimated_effort && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Estimated Effort</h4>
                                                                    <p className="m-0">{todo.estimated_effort}</p>
                                                                </div>
                                                            )}

                                                            {todo.files_affected && todo.files_affected.length > 0 && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Files Affected</h4>
                                                                    <ul className="m-0 pl-3">
                                                                        {todo.files_affected.map((file, index) => (
                                                                            <li key={index}>{file}</li>
                                                                        ))}
                                                                    </ul>
                                                                </div>
                                                            )}

                                                            {todo.function_signatures && todo.function_signatures.length > 0 && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Function Signatures</h4>
                                                                    <ul className="m-0 pl-3">
                                                                        {todo.function_signatures.map((signature, index) => (
                                                                            <li key={index}><code>{signature}</code></li>
                                                                        ))}
                                                                    </ul>
                                                                </div>
                                                            )}

                                                            {todo.testing_requirements && todo.testing_requirements.length > 0 && (
                                                                <div className="mb-3">
                                                                    <h4 className="m-0 mb-2">Testing Requirements</h4>
                                                                    <ul className="m-0 pl-3">
                                                                        {todo.testing_requirements.map((requirement, index) => (
                                                                            <li key={index}>{requirement}</li>
                                                                        ))}
                                                                    </ul>
                                                                </div>
                                                            )}

                                                            {todo.commit_id && todo.commit_id.length > 0 && (
                                                                <div className="mb-3">
                                                                    <div className="mb-3">
                                                                        <h4 className="m-0 mb-2">Commit Id</h4>
                                                                        <p className="m-0">{todo.commit_id}</p>
                                                                    </div>
                                                                </div>
                                                            )}

                                                            {todo.status && todo.status.length > 0 && (
                                                                <div className="mb-3">
                                                                    <div className="mb-3">
                                                                        <h4 className="m-0 mb-2">Status</h4>
                                                                        <p className="m-0">{todo.status}</p>
                                                                    </div>
                                                                </div>
                                                            )}


                                                        </div>
                                                    </AccordionTab>
                                                ))}
                                            </Accordion>
                                        </div>
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
