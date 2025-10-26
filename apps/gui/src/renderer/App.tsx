import { useState, useEffect } from "react";
import { HDQLQueryBuilder } from "./components/HDQLQueryBuilder";
import { ResultsViewer } from "./components/ResultsViewer";

export function App() {
  const [results, setResults] = useState<any[]>([]);
  const [metadata, setMetadata] = useState<any>(null);
  const [collections, setCollections] = useState<any[]>([]);
  const [portals, setPortals] = useState<any[]>([]);

  useEffect(() => {
    // Load collections and portals on mount
    window.electron.getCollections().then(setCollections);
    window.electron.getPortals().then(setPortals);
  }, []);

  const handleExecuteQuery = async (queryText: string) => {
    try {
      const result = await window.electron.executeHDQL(queryText);
      setResults(result.results || []);
      setMetadata(result.metadata || {});
    } catch (error) {
      console.error("Query execution failed:", error);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>Hive-News GUI</h1>
        <div className="header-actions">
          <select>
            {portals.map((portal) => (
              <option key={portal.id} value={portal.id}>
                {portal.name}
              </option>
            ))}
          </select>
          <button>Settings</button>
        </div>
      </header>

      <div className="app-body">
        <nav className="sidebar">
          <h3>Collections</h3>
          <ul className="collections-list">
            {collections.map((collection) => (
              <li key={collection.id}>
                {collection.name} ({collection.count})
              </li>
            ))}
          </ul>

          <h3>Saved Queries</h3>
          <ul className="saved-queries">
            <li>Top Trending Articles</li>
            <li>Semantic Search</li>
            <li>Portal Statistics</li>
          </ul>
        </nav>

        <main className="content">
          <div className="query-section">
            <HDQLQueryBuilder onExecute={handleExecuteQuery} />
          </div>

          <div className="results-section">
            <ResultsViewer results={results} metadata={metadata} />
          </div>
        </main>
      </div>
    </div>
  );
}
