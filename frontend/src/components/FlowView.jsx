import {useState, useEffect, useCallback, useRef} from 'react';
import ReactFlow, {
    Background,
    Controls,
    useNodesState,
    useEdgesState,
    MarkerType
} from 'reactflow';
import 'reactflow/dist/style.css';
import './FlowView.css';

const FlowView = () => {
    const [blocks, setBlocks] = useState([]);
    const [loading, setLoading] = useState(true);
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    // Create a ref to store the current blocks data
    const blocksRef = useRef([]);

    // Update the ref whenever blocks changes
    useEffect(() => {
        blocksRef.current = blocks;
    }, [blocks]);

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
                    processBlocksData(data);
                }
            } catch (error) {
                console.error('Error checking blocks configuration:', error);
            }
        }, 5000); // Check every 5 seconds

        // Clean up the interval when the component unmounts
        return () => clearInterval(checkConfigInterval);
    }, []);

    const fetchBlocks = async () => {
        try {
            setLoading(true);
            const response = await fetch('/api/blocks');
            if (!response.ok) {
                throw new Error('Failed to fetch blocks');
            }
            const data = await response.json();
            setBlocks(data);
            processBlocksData(data);
        } catch (error) {
            console.error('Error fetching blocks:', error);
        } finally {
            setLoading(false);
        }
    };

    const processBlocksData = (blocksData) => {
        // Create nodes from blocks
        const flowNodes = blocksData.map((block, index) => {
            // Position nodes in a grid layout
            const posX = (index % 3) * 300 + 50;
            const posY = Math.floor(index / 3) * 200 + 50;

            return {
                id: block.block_id,
                data: {
                    label: (
                        <div>
                            <div className="flow-node-title">{block.block_id}</div>
                            <div className="flow-node-io">
                                <div>Inputs: {block.inputs.map(input => input.name).join(', ')}</div>
                                <div>Outputs: {block.outputs.map(output => output.name).join(', ')}</div>
                            </div>
                        </div>
                    )
                },
                position: {x: posX, y: posY},
                style: {
                    background: '#1e1e1e',
                    color: '#fff',
                    border: '1px solid #00f2ff',
                    boxShadow: '0 0 10px rgba(0, 242, 255, 0.5)',
                    borderRadius: '5px',
                    padding: '5px',
                    width: 250,
                },
            };
        });

        // Create edges from connections
        const flowEdges = [];

        blocksData.forEach(block => {
            // Process output connections
            block.connections.output_connections.forEach(conn => {
                flowEdges.push({
                    id: `${block.block_id}-${conn.to_module}-${conn.unique_id}`,
                    source: block.block_id,
                    target: conn.to_module,
                    label: conn.input_type,
                    type: 'default',
                    animated: true,
                    style: {stroke: '#00f2ff'},
                    labelStyle: {
                        fill: '#fff',
                        fontWeight: 'normal',
                        background: 'transparent',
                        border: '1px solid rgba(255, 255, 255, 0.3)',
                        borderRadius: '4px',
                        padding: '1px 2px',
                    },
                    labelBgStyle: {fill: 'transparent'},
                    labelBgPadding: [2, 1],
                    markerEnd: {
                        type: MarkerType.ArrowClosed,
                        width: 20,
                        height: 20,
                        color: '#00f2ff',
                    },
                });
            });
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
                nodePositions[node.id] = {x: posX, y: posY};
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

    if (loading) {
        return <div>Loading blocks...</div>;
    }

    return (
        <div className="flow-container">
            <div className="flow-header">
                <button onClick={() => onLayout('TB')} className="layout-button">
                    Auto Layout
                </button>
            </div>
            <div className="flow-canvas">
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    onNodesChange={onNodesChange}
                    onEdgesChange={onEdgesChange}
                    fitView
                    className="dark-flow"
                >
                    <Controls/>
                    <Background variant="dots" gap={12} size={1} color="#444"/>
                </ReactFlow>
            </div>
        </div>
    );
};

export default FlowView;
