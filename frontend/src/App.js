import React, { useState, useEffect, useCallback } from "react";
import { BrowserRouter as Router, Route, Routes, Link } from "react-router-dom";
import { useParams } from "react-router-dom";

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
            <Link 
              to={`/all-registries/${encodeURIComponent(reg.doc_id)}`} 
              key={index} 
              style={{ textDecoration: "none", color: "inherit" }}
            >
              <div
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
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

function RegistryEntries() {
  const { doc_id } = useParams(); // get doc_id from URL
  const [showModal, setShowModal] = useState(false);
  const [entryData, setEntryData] = useState("");
  const [entries, setEntries] = useState([]); // to store fetched entries

  // Use useCallback to memoize the fetchEntries function
  const fetchEntries = useCallback(async () => {
    try {
      const urlEncodedData = new URLSearchParams();
      urlEncodedData.append("registry_id", doc_id.replace(/,/g, ""));

      const response = await fetch("http://localhost:4000/display_entries", {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: urlEncodedData.toString(),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText);
      }

      const result = await response.json();
      setEntries(result.entries); // save to state
      console.log("Fetched entries:", result.entries);
    } catch (error) {
      alert("Failed to fetch entries: " + error.message);
    }
  }, [doc_id]); // only re-run fetchEntries when doc_id changes

  // useEffect now depends on fetchEntries
  useEffect(() => {
    fetchEntries();
  }, [fetchEntries]); // No warning now because of useCallback

  const handleAddEntry = async () => {
    try {
      const urlEncodedData = new URLSearchParams();
      urlEncodedData.append("registry_id", doc_id.replace(/,/g, ""));
      urlEncodedData.append("entry_data", entryData);

      const response = await fetch("http://localhost:4000/add_entry", {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: urlEncodedData.toString(),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText);
      }

      const result = await response.text();
      alert(result);
      setShowModal(false);
      setEntryData("");

      // Call fetchEntries again after adding a new entry
      fetchEntries(); // Ensure entries are refreshed
    } catch (error) {
      alert("Failed to add entry: " + error.message);
    }
  };

  const handleDeleteEntry = async (entryId) => {
    try {
      const urlEncodedData = new URLSearchParams();
      urlEncodedData.append("registry_id", doc_id.replace(/,/g, ""));
      urlEncodedData.append("entry_id", entryId);

      const response = await fetch("http://localhost:4000/delete_entry", {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: urlEncodedData.toString(),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText);
      }

      const result = await response.text();
      alert(result);

      // Call fetchEntries again after deleting an entry
      fetchEntries(); // Ensure entries are refreshed
    } catch (error) {
      alert("Failed to delete entry: " + error.message);
    }
  };

  return (
    <div style={{ padding: "2rem" }}>
      <h2>All Entries</h2>
      <p>Registry Doc ID: {doc_id.replace(/,/g, "")}</p>

      <button onClick={() => setShowModal(true)} style={{ marginBottom: "1rem" }}>
        Add Entry
      </button>

      {showModal && (
        <div
          style={{
            position: "fixed",
            top: "30%",
            left: "30%",
            right: "30%",
            backgroundColor: "white",
            padding: "2rem",
            boxShadow: "0 2px 8px rgba(0,0,0,0.26)",
            zIndex: 1000,
          }}
        >
          <h3>Add New Entry</h3>
          <textarea
            placeholder="Enter entry JSON"
            rows="10"
            cols="50"
            value={entryData}
            onChange={(e) => setEntryData(e.target.value)}
          />
          <br />
          <button onClick={handleAddEntry} style={{ marginRight: "1rem" }}>
            Submit
          </button>
          <button onClick={() => setShowModal(false)}>Cancel</button>
        </div>
      )}

      <div style={{ marginTop: "2rem" }}>
        <h3>Entries:</h3>
        {entries.length === 0 ? (
          <p>No entries found.</p>
        ) : (
          entries.map((entry, index) => (
            <div
              key={index}
              style={{ padding: "1rem", border: "1px solid black", marginBottom: "1rem" }}
            >
              {Object.entries(entry).map(([key, value]) => (
                <p key={key}>
                  <strong>{key}</strong>: {JSON.stringify(value)}
                </p>
              ))}
              <button onClick={() => handleDeleteEntry(entry.id)}>Delete</button>
            </div>
          ))
        )}
      </div>
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
        <Route path="/all-registries/:doc_id" element={<RegistryEntries />} />
        <Route path="/archive" element={<ArchiveRegistry />} />
      </Routes>
    </Router>
  );
}

export default App;
