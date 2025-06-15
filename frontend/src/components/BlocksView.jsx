import { useState, useEffect } from 'react';
import { Card } from 'primereact/card';
import { InputTextarea } from 'primereact/inputtextarea';
import { Button } from 'primereact/button';
import { Divider } from 'primereact/divider';
import { Chip } from 'primereact/chip';
import { Panel } from 'primereact/panel';
import './BlocksView.css';

const BlocksView = () => {
  const [blocks, setBlocks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [editingDescription, setEditingDescription] = useState({});

  useEffect(() => {
    fetchBlocks();
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

  const saveDescription = (blockName) => {
    setBlocks(blocks.map(block => {
      if (block.name === blockName) {
        return { ...block, description: editingDescription[blockName] };
      }
      return block;
    }));
    // In a real application, you would also send an API request to update the description on the server
  };

  const startEditing = (block) => {
    setEditingDescription({
      ...editingDescription,
      [block.name]: block.description
    });
  };

  if (loading) {
    return <div>Loading blocks...</div>;
  }

  return (
    <div className="blocks-container">
      <h2>Blocks</h2>
      <div className="grid">
        {blocks.map((block) => (
          <div key={block.name} className="col-12 md:col-6 lg:col-4 p-2">
            <Card 
              title={block.name} 
              className="block-card"
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
                          icon="pi pi-check" 
                          className="p-button-sm p-button-success" 
                          onClick={() => saveDescription(block.name)}
                          tooltip="Save description"
                        />
                        <Button 
                          icon="pi pi-times" 
                          className="p-button-sm p-button-danger ml-2" 
                          onClick={() => setEditingDescription({...editingDescription, [block.name]: undefined})}
                          tooltip="Cancel"
                        />
                      </div>
                    </div>
                  ) : (
                    <>
                      <span className="mr-2">{block.description}</span>
                      <Button 
                        icon="pi pi-pencil" 
                        className="p-button-sm p-button-text" 
                        onClick={() => startEditing(block)}
                        tooltip="Edit description"
                      />
                    </>
                  )}
                </div>
              }
            >
              <Divider />

              <Panel header="Inputs & Outputs" toggleable>
                <div className="mb-3">
                  <h4>Inputs:</h4>
                  <div className="flex flex-wrap gap-2">
                    {block.inputs.map((input, index) => (
                      <Chip key={index} label={input} />
                    ))}
                  </div>
                </div>

                <div>
                  <h4>Outputs:</h4>
                  <div className="flex flex-wrap gap-2">
                    {block.outputs.map((output, index) => (
                      <Chip key={index} label={output} />
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

              <Panel header="Todo List" toggleable>
                {block.todo_list.length > 0 ? (
                  <ul className="m-0 p-0 list-none">
                    {block.todo_list.map((todo, index) => (
                      <li key={index} className="mb-2 flex align-items-center">
                        <i className="pi pi-check-circle mr-2"></i>
                        {todo}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No todo items</p>
                )}
              </Panel>
            </Card>
          </div>
        ))}
      </div>
    </div>
  );
};

export default BlocksView;
