import React, { useState, useEffect } from "react";
import { BrowserRouter as Router, Route, Routes, Link } from "react-router-dom";

function Home() {
  return (
    <div style={{ padding: "1rem" }}>
      <h1>Registry UI</h1>
      <nav>
        <Link to="/create">Create New Registry</Link> |{" "}
        <Link to="/all-registries">See All Registries</Link> |{" "}
        <Link to="/archive">Archive Registry</Link>
      </nav>
    </div>
  );
}

function CreateRegistry() {
  const [registryName, setRegistryName] = useState("");
  const [schema, setSchema] = useState("");
  const [file, setFile] = useState(null);
  const [message, setMessage] = useState("");

  const handleCreate = async () => {
    if (!registryName || !schema || !file) {
      alert("Please fill in all fields.");
      return;
    }

    const formData = new FormData();
    formData.append("registry_name", registryName);
    formData.append("schema", schema);
    formData.append("file", file);

    try {
      const response = await fetch("http://localhost:4000/create_registry", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const errMsg = await response.text();
        throw new Error(errMsg);
      }

      const result = await response.json();
      setMessage(`✅ Registry created! Doc ID: ${result}`);
    } catch (err) {
      setMessage(`❌ Error: ${err.message}`);
    }
  };

  return (
    <div style={{ padding: "1rem" }}>
      <h2>Create New Registry</h2>
      <div>
        <label>Registry Name:</label>
        <input
          type="text"
          placeholder="Enter registry name"
          value={registryName}
          onChange={(e) => setRegistryName(e.target.value)}
        />
      </div>
      <div>
        <label>Schema (as JSON):</label>
        <input
          type="text"
          placeholder="Enter registry schema"
          value={schema}
          onChange={(e) => setSchema(e.target.value)}
        />
      </div>
      <div>
        <label>Upload File:</label>
        <input
          type="file"
          onChange={(e) => setFile(e.target.files[0])}
        />
      </div>
      <button onClick={handleCreate}>Create</button>

      {message && <p style={{ marginTop: "1rem" }}>{message}</p>}
    </div>
  );
}

function SeeAllRegistries() {
  const [registries, setRegistries] = useState([]);

  useEffect(() => {
    fetch("http://localhost:4000/all_registries")
      .then((res) => res.json())
      .then((data) => setRegistries(data.registries || []))
      .catch((err) => console.error("Error fetching registries:", err));
  }, []);

  return (
    <div style={{ padding: "2rem" }}>
      <h2>All Registries</h2>
      {registries.length === 0 ? (
        <p>No registries found.</p>
      ) : (
        <div style={{ display: "flex", flexWrap: "wrap", gap: "1rem" }}>
          {registries.map((reg, index) => (
            <div
              key={index}
              style={{
                border: "1px solid #ccc",
                borderRadius: "8px",
                padding: "1rem",
                minWidth: "300px",
                backgroundColor: "#f9f9f9",
              }}
            >
              <h3>{reg.registry_name}</h3>
              <pre style={{ whiteSpace: "pre-wrap", wordWrap: "break-word" }}>
                {/* <strong>Schema:</strong> {JSON.stringify(reg.schema, null, 2)} */}
                <strong>Schema:</strong> {reg.schema}
              </pre>
              {reg.file && (
                <div>
                  <strong>File:</strong>
                  <pre style={{ whiteSpace: "pre-wrap", wordWrap: "break-word" }}>
                    {JSON.stringify(reg.file, null, 2)}
                  </pre>
                </div>
              )}
              {reg.archived !== undefined && (
                <p>
                  <strong>Archived:</strong> {reg.archived ? "Yes" : "No"}
                </p>
              )}
              <p><strong>Doc ID:</strong> {reg.doc_id}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}


function ArchiveRegistry() {

  const [registryName, setRegistryName] = useState("");

  const handleArchive = async (e) => {
    e.preventDefault();

    if (!registryName.trim()) {
      alert("Please enter a registry name.");
      return;
    }

    const formData = new URLSearchParams();
    formData.append("registry_name", registryName);

    try {
      const response = await fetch("http://localhost:4000/archive", {  
        method: "POST",
        body: formData,
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
      });

      const result = await response.text();

      if (response.ok) {
        alert(`Registry archived successfully. Hash: ${result}`);
      } else {
        alert(`Failed to archive registry: ${result}`);
      }
    } catch (error) {
      console.error("Error archiving registry:", error);
      alert("An error occurred while archiving registry.");
    }
  };

  return (
    <div style={{ padding: "1rem" }}>
      <h2>Archive Registry</h2>
      <div>
        <label>Registry Name:</label>
        <input
          type="text"
          placeholder="Enter registry name"
          value={registryName}
          onChange={(e) => setRegistryName(e.target.value)}
        />
      </div>
      <button onClick={handleArchive}>Archive</button>
    </div>
  );
}

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/create" element={<CreateRegistry />} />
        <Route path="/all-registries" element={<SeeAllRegistries />} />
        <Route path="/archive" element={<ArchiveRegistry />} />
      </Routes>
    </Router>
  );
}

export default App;
