import React from "react";
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
  return (
    <div style={{ padding: "1rem" }}>
      <h2>Create New Registry</h2>
      <div>
        <label>Registry Name:</label>
        <input type="text" placeholder="Enter registry name" />
      </div>
      <div>
        <label>Schema:</label>
        <input type="text" placeholder="Enter registry schema" />
      </div>
      <div>
        <label>Upload File:</label>
        <input type="file" />
      </div>
      <button>Create</button>
    </div>
  );
}

function SeeAllRegistries() {
  return (
    <div style={{ padding: "1rem" }}>
      <h2>All Registries</h2>
      <p>This is a placeholder page. We'll add functionality later.</p>
    </div>
  );
}

function ArchiveRegistry() {
  return (
    <div style={{ padding: "1rem" }}>
      <h2>Archive Registry</h2>
      <div>
        <label>Registry Name:</label>
        <input type="text" placeholder="Enter registry name" />
      </div>
      <button>Archive</button>
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
