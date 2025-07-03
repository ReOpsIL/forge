/**
 * Task sorting utilities for maintaining stable task ordering
 */

/**
 * Sorts tasks stably using task_id as the primary key
 * This ensures consistent ordering across refreshes
 * @param {Array} tasks - Array of task objects
 * @returns {Array} - Sorted array of tasks
 */
export const sortTasksStably = (tasks) => {
    if (!Array.isArray(tasks)) {
        return [];
    }

    return [...tasks].sort((a, b) => {
        // Primary sort by task_id for stable ordering
        if (a.task_id && b.task_id) {
            return a.task_id.localeCompare(b.task_id);
        }

        // Fallback to description if task_id is not available
        if (a.description && b.description) {
            return a.description.localeCompare(b.description);
        }

        // Final fallback - maintain original order
        return 0;
    });
};

/**
 * Maintains task order when merging current tasks with new tasks
 * Preserves existing order while adding new tasks in stable positions
 * @param {Array} currentTasks - Current tasks array
 * @param {Array} newTasks - New tasks array from server
 * @returns {Array} - Merged tasks with stable ordering
 */
export const maintainTaskOrder = (currentTasks, newTasks) => {
    if (!Array.isArray(currentTasks) || !Array.isArray(newTasks)) {
        return sortTasksStably(newTasks || []);
    }

    // Create a map of current tasks by task_id for quick lookup
    const currentTasksMap = new Map();
    currentTasks.forEach((task, index) => {
        if (task.task_id) {
            currentTasksMap.set(task.task_id, {...task, originalIndex: index});
        }
    });

    // Separate existing tasks (that maintain their order) from new tasks
    const existingTasks = [];
    const newTasksOnly = [];

    newTasks.forEach(task => {
        if (task.task_id && currentTasksMap.has(task.task_id)) {
            // This is an existing task - preserve its order info
            const currentTask = currentTasksMap.get(task.task_id);
            existingTasks.push({
                ...task,
                originalIndex: currentTask.originalIndex
            });
        } else {
            // This is a new task
            newTasksOnly.push(task);
        }
    });

    // Sort existing tasks by their original index to maintain order
    existingTasks.sort((a, b) => a.originalIndex - b.originalIndex);

    // Sort new tasks stably
    const sortedNewTasks = sortTasksStably(newTasksOnly);

    // Combine existing tasks (in original order) with new tasks (in stable order)
    return [...existingTasks, ...sortedNewTasks].map(task => {
        // Remove the originalIndex property we added for sorting
        const {originalIndex, ...cleanTask} = task;
        return cleanTask;
    });
};

/**
 * Converts todo_list object to array with stable sorting
 * @param {Object} todoList - Todo list object with task_id as keys
 * @returns {Array} - Stable sorted array of tasks
 */
export const todoListToSortedArray = (todoList) => {
    if (!todoList || typeof todoList !== 'object') {
        return [];
    }

    const tasks = Object.values(todoList);
    return sortTasksStably(tasks);
};

/**
 * Determines if task reordering has occurred by comparing two task arrays
 * @param {Array} oldTasks - Previous tasks array
 * @param {Array} newTasks - New tasks array
 * @returns {boolean} - True if reordering occurred
 */
export const hasTaskReorderingOccurred = (oldTasks, newTasks) => {
    if (!Array.isArray(oldTasks) || !Array.isArray(newTasks)) {
        return false;
    }

    if (oldTasks.length !== newTasks.length) {
        return false; // This is a content change, not reordering
    }

    // Check if the order of task_ids has changed
    for (let i = 0; i < oldTasks.length; i++) {
        if (oldTasks[i].task_id !== newTasks[i].task_id) {
            return true;
        }
    }

    return false;
};