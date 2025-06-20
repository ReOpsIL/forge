import { useState, useEffect, useCallback } from 'react';
import ReactFlow, {
    Background,
    Controls,
    useNodesState,
    useEdgesState,
    MarkerType
} from 'reactflow';
import 'reactflow/dist/style.css';
import './FlowView.css';
import { Dialog } from 'primereact/dialog';
import { Button } from 'primereact/button';

const DependencyTreeView = ({ blockId, visible, onHide }) => {
    const [loading, setLoading] = useState(true);
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    useEffect(() => {
        if (visible && blockId) {
            fetchDependencies();
        }
    }, [visible, blockId]);

    const fetchDependencies = async () => {
        try {
            setLoading(true);
            const response = await fetch(`/api/blocks/${blockId}/dependencies`);
            if (!response.ok) {
                throw new Error('Failed to fetch dependencies');
            }
            const data = await response.json();
            processDependenciesData(data);
        } catch (error) {
            console.error('Error fetching dependencies:', error);
        } finally {
            setLoading(false);
        }
    };

    const processDependenciesData = (data) => {
        // Create nodes from tasks
        const flowNodes = [];
        const flowEdges = [];
        const nodePositions = {};
        
        // First pass: create nodes for all tasks
        data.tasks.forEach((task, index) => {
            // Position nodes in a grid layout
            const posX = (index % 3) * 300 + 50;
            const posY = Math.floor(index / 3) * 200 + 50;
            
            nodePositions[task.task_id] = { x: posX, y: posY };
            
            flowNodes.push({
                id: task.task_id,
                data: {
                    label: (
                        <div>
                            <div className="flow-node-title">{task.task_id}</div>
                            <div className="flow-node-description">{task.description}</div>
                        </div>
                    )
                },
                position: { x: posX, y: posY },
                style: {
                    background: '#1e1e1e',
                    color: '#fff',
                    border: '1px solid #00f2ff',
                    boxShadow: '0 0 10px rgba(0, 242, 255, 0.5)',
                    borderRadius: '5px',
                    padding: '5px',
                    width: 250,
                },
            });
        });
        
        // Second pass: create edges for dependencies
        data.tasks.forEach(task => {
            if (task.dependencies && task.dependencies.length > 0) {
                task.dependencies.forEach(dependencyId => {
                    flowEdges.push({
                        id: `${dependencyId}-${task.task_id}`,
                        source: dependencyId,
                        target: task.task_id,
                        type: 'default',
                        animated: true,
                        style: { stroke: '#00f2ff' },
                        markerEnd: {
                            type: MarkerType.ArrowClosed,
                            width: 20,
                            height: 20,
                            color: '#00f2ff',
                        },
                    });
                });
            }
        });

        setNodes(flowNodes);
        setEdges(flowEdges);
    };

    const onLayout = useCallback(
        (direction) => {
            // This is a simple layout function
            // In a real application, you might want to use a more sophisticated layout algorithm
            const nodePositions = {};
            nodes.forEach((node, index) => {
                const posX = (index % 3) * 300 + 50;
                const posY = Math.floor(index / 3) * 200 + 50;
                nodePositions[node.id] = { x: posX, y: posY };
            });

            setNodes((nds) =>
                nds.map((node) => ({
                    ...node,
                    position: nodePositions[node.id],
                }))
            );
        },
        [nodes, setNodes]
    );

    return (
        <Dialog
            header={`Dependency Tree for Block ${blockId}`}
            visible={visible}
            style={{ width: '80vw', height: '80vh' }}
            onHide={onHide}
            maximizable
            modal
        >
            <div className="flow-container" style={{ height: 'calc(100% - 50px)' }}>
                <div className="flow-header">
                    <Button onClick={() => onLayout('TB')} className="layout-button">
                        Auto Layout
                    </Button>
                </div>
                <div className="flow-canvas" style={{ height: 'calc(100% - 50px)' }}>
                    {loading ? (
                        <div>Loading dependencies...</div>
                    ) : (
                        <ReactFlow
                            nodes={nodes}
                            edges={edges}
                            onNodesChange={onNodesChange}
                            onEdgesChange={onEdgesChange}
                            fitView
                            className="dark-flow"
                        >
                            <Controls />
                            <Background variant="dots" gap={12} size={1} color="#444" />
                        </ReactFlow>
                    )}
                </div>
            </div>
        </Dialog>
    );
};

export default DependencyTreeView;