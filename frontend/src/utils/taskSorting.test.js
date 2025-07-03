/**
 * Basic tests for task sorting utilities
 * Run with: node taskSorting.test.js
 */

import {maintainTaskOrder, sortTasksStably, todoListToSortedArray} from './taskSorting.js';

// Simple test runner
const test = (name, fn) => {
    try {
        fn();
        console.log(`✓ ${name}`);
    } catch (error) {
        console.error(`✗ ${name}: ${error.message}`);
    }
};

const assertEqual = (actual, expected, message = '') => {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}. ${message}`);
    }
};

// Test data
const testTasks = [
    {task_id: 'task_3', description: 'Task 3'},
    {task_id: 'task_1', description: 'Task 1'},
    {task_id: 'task_2', description: 'Task 2'}
];

const testTodoList = {
    'task_3': {task_id: 'task_3', description: 'Task 3'},
    'task_1': {task_id: 'task_1', description: 'Task 1'},
    'task_2': {task_id: 'task_2', description: 'Task 2'}
};

// Run tests
test('sortTasksStably sorts by task_id', () => {
    const result = sortTasksStably(testTasks);
    const expected = [
        {task_id: 'task_1', description: 'Task 1'},
        {task_id: 'task_2', description: 'Task 2'},
        {task_id: 'task_3', description: 'Task 3'}
    ];
    assertEqual(result, expected);
});

test('sortTasksStably handles empty array', () => {
    const result = sortTasksStably([]);
    assertEqual(result, []);
});

test('sortTasksStably handles null/undefined', () => {
    const result1 = sortTasksStably(null);
    const result2 = sortTasksStably(undefined);
    assertEqual(result1, []);
    assertEqual(result2, []);
});

test('todoListToSortedArray converts object to sorted array', () => {
    const result = todoListToSortedArray(testTodoList);
    const expected = [
        {task_id: 'task_1', description: 'Task 1'},
        {task_id: 'task_2', description: 'Task 2'},
        {task_id: 'task_3', description: 'Task 3'}
    ];
    assertEqual(result, expected);
});

test('maintainTaskOrder preserves existing order', () => {
    const currentTasks = [
        {task_id: 'task_2', description: 'Task 2'},
        {task_id: 'task_1', description: 'Task 1'}
    ];
    const newTasks = [
        {task_id: 'task_3', description: 'Task 3'},
        {task_id: 'task_1', description: 'Task 1'},
        {task_id: 'task_2', description: 'Task 2'}
    ];

    const result = maintainTaskOrder(currentTasks, newTasks);

    // Should maintain order of existing tasks (task_2, task_1) and add new task (task_3) at the end
    const expected = [
        {task_id: 'task_2', description: 'Task 2'},
        {task_id: 'task_1', description: 'Task 1'},
        {task_id: 'task_3', description: 'Task 3'}
    ];
    assertEqual(result, expected);
});

console.log('Running task sorting tests...');
console.log('');