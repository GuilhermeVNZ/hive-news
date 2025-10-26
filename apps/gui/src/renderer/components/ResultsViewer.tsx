import { useEffect, useState } from "react";

interface QueryResult {
  id: string;
  [key: string]: any;
}

interface ResultsViewerProps {
  results: QueryResult[];
  metadata?: {
    executionTime: number;
    resultCount: number;
    query?: string;
  };
}

export function ResultsViewer({ results, metadata }: ResultsViewerProps) {
  const [selectedResult, setSelectedResult] = useState<QueryResult | null>(
    null
  );

  if (results.length === 0) {
    return (
      <div className="results-viewer empty">
        <div className="empty-state">
          <h3>No Results</h3>
          <p>Execute a query to see results here</p>
        </div>
        {metadata && (
          <div className="query-metadata">
            <span>Execution time: {metadata.executionTime}ms</span>
          </div>
        )}
      </div>
    );
  }

  const columns = results.length > 0 ? Object.keys(results[0]) : [];

  return (
    <div className="results-viewer">
      <div className="results-header">
        <h3>
          Results ({metadata?.resultCount || results.length} items)
        </h3>
        {metadata && (
          <div className="metadata">
            <span>Execution time: {metadata.executionTime}ms</span>
          </div>
        )}
      </div>

      <div className="results-grid">
        <table>
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col}>{col}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {results.map((result, index) => (
              <tr
                key={result.id || index}
                onClick={() => setSelectedResult(result)}
                className={
                  selectedResult?.id === result.id ? "selected" : ""
                }
              >
                {columns.map((col) => (
                  <td key={col}>
                    {typeof result[col] === "object"
                      ? JSON.stringify(result[col])
                      : String(result[col])}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {selectedResult && (
        <div className="result-detail">
          <h4>Result Details</h4>
          <pre>{JSON.stringify(selectedResult, null, 2)}</pre>
          <button onClick={() => setSelectedResult(null)}>Close</button>
        </div>
      )}
    </div>
  );
}
