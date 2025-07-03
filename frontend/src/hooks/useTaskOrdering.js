import {useEffect, useMemo, useRef, useState} from 'react';
import {maintainTaskOrder, sortTasksStably, todoListToSortedArray} from '../utils/taskSorting';

/**
 * Custom hook for managing stable task ordering
 * @param {Object} todoList - The todo_list object from the block
 * @returns {Object} - Object containing sorted tasks and ordering utilities
 */
export const useStableTaskOrder = (todoList) => {
    const [stableTasks, setStableTasks] = useState([]);
    const previousTasksRef = useRef([]);

    // Convert todo_list to sorted array with memoization
    const sortedTasks = useMemo(() => {
        return todoListToSortedArray(todoList);
    }, [todoList]);

    // Maintain stable ordering when tasks change
    useEffect(() => {
        const currentTasks = previousTasksRef.current;
        const newTasks = sortedTasks;

        if (currentTasks.length === 0) {
            // First load - use stable sorting
            const initialSortedTasks = sortTasksStably(newTasks);
            setStableTasks(initialSortedTasks);
            previousTasksRef.current = initialSortedTasks;
        } else {
            // Subsequent updates - maintain order
            const maintainedTasks = maintainTaskOrder(currentTasks, newTasks);
            setStableTasks(maintainedTasks);
            previousTasksRef.current = maintainedTasks;
        }
    }, [sortedTasks]);

    // Force refresh stable ordering (for manual refresh)
    const refreshStableOrder = () => {
        const freshSortedTasks = sortTasksStably(sortedTasks);
        setStableTasks(freshSortedTasks);
        previousTasksRef.current = freshSortedTasks;
    };

    return {
        stableTasks,
        refreshStableOrder,
        taskCount: stableTasks.length
    };
};

/**
 * Custom hook for managing task ordering across all blocks
 * @param {Array} blocks - Array of block objects
 * @returns {Object} - Object with stable task orderings for all blocks
 */
export const useBlockTaskOrdering = (blocks) => {
    const [stableBlockTasks, setStableBlockTasks] = useState({});
    const previousBlockTasksRef = useRef({});

    useEffect(() => {
        if (!Array.isArray(blocks)) {
            return;
        }

        const newStableBlockTasks = {};

        blocks.forEach(block => {
            const blockId = block.block_id;
            const currentTasks = previousBlockTasksRef.current[blockId] || [];
            const newTasks = todoListToSortedArray(block.todo_list);

            if (currentTasks.length === 0) {
                // First load for this block
                newStableBlockTasks[blockId] = sortTasksStably(newTasks);
            } else {
                // Maintain order for this block
                newStableBlockTasks[blockId] = maintainTaskOrder(currentTasks, newTasks);
            }
        });

        setStableBlockTasks(newStableBlockTasks);
        previousBlockTasksRef.current = newStableBlockTasks;
    }, [blocks]);

    // Get stable tasks for a specific block
    const getStableTasksForBlock = (blockId) => {
        return stableBlockTasks[blockId] || [];
    };

    // Force refresh for a specific block
    const refreshBlockOrder = (blockId) => {
        const block = blocks.find(b => b.block_id === blockId);
        if (block) {
            const freshTasks = sortTasksStably(todoListToSortedArray(block.todo_list));
            setStableBlockTasks(prev => ({
                ...prev,
                [blockId]: freshTasks
            }));
            previousBlockTasksRef.current = {
                ...previousBlockTasksRef.current,
                [blockId]: freshTasks
            };
        }
    };

    // Force refresh for all blocks
    const refreshAllBlockOrders = () => {
        const newStableBlockTasks = {};
        blocks.forEach(block => {
            const blockId = block.block_id;
            newStableBlockTasks[blockId] = sortTasksStably(todoListToSortedArray(block.todo_list));
        });
        setStableBlockTasks(newStableBlockTasks);
        previousBlockTasksRef.current = newStableBlockTasks;
    };

    return {
        getStableTasksForBlock,
        refreshBlockOrder,
        refreshAllBlockOrders,
        stableBlockTasks
    };
};