import { useState } from "react";

interface HDQLQuery {
  from: string;
  where: string[];
  select: string[];
  orderBy: { field: string; direction: "ASC" | "DESC" }[];
  limit: number;
  search?: {
    query: string;
    fields: string[];
  };
  vectorSearch?: {
    field: string;
    query: string;
    threshold: number;
  };
}

interface HDQLQueryBuilderProps {
  onExecute?: (queryText: string) => void;
}

export function HDQLQueryBuilder({ onExecute }: HDQLQueryBuilderProps) {
  const [query, setQuery] = useState<HDQLQuery>({
    from: "",
    where: [],
    select: ["*"],
    orderBy: [],
    limit: 20,
  });

  const [queryText, setQueryText] = useState("");
  const [isValid, setIsValid] = useState(true);

  // Translate visual query to HDQL text
  const generateHDQL = (q: HDQLQuery): string => {
    let hdql = `FROM ${q.from}\n`;

    if (q.select.length > 0) {
      hdql += `SELECT ${q.select.join(", ")}\n`;
    }

    if (q.where.length > 0) {
      hdql += `WHERE ${q.where.join(" AND ")}\n`;
    }

    if (q.search) {
      hdql += `SEARCH "${q.search.query}" IN [${q.search.fields.join(", ")}]\n`;
    }

    if (q.vectorSearch) {
      hdql += `WHERE vector_similarity(${q.vectorSearch.field}, "${q.vectorSearch.query}") > ${q.vectorSearch.threshold}\n`;
    }

    if (q.orderBy.length > 0) {
      const orderClauses = q.orderBy.map(
        (o) => `${o.field} ${o.direction}`
      );
      hdql += `ORDER BY ${orderClauses.join(", ")}\n`;
    }

    if (q.limit > 0) {
      hdql += `LIMIT ${q.limit}\n`;
    }

    return hdql;
  };

  return (
    <div className="query-builder">
      <div className="query-builder-header">
        <h2>HDQL Query Builder</h2>
        <div className="query-actions">
          <button onClick={() => setQueryText(generateHDQL(query))}>
            Generate HDQL
          </button>
          <button
            disabled={!isValid}
            onClick={() => onExecute?.(queryText || generateHDQL(query))}
          >
            Execute
          </button>
          <button onClick={() => setQuery({ ...query, where: [], select: ["*"], orderBy: [], limit: 20 })}>
            Clear
          </button>
        </div>
      </div>

      <div className="query-builder-body">
        {/* Collection Selection */}
        <div className="builder-section">
          <label>FROM Collection:</label>
          <input
            type="text"
            value={query.from}
            onChange={(e) => setQuery({ ...query, from: e.target.value })}
            placeholder="e.g., articles, news-content"
          />
        </div>

        {/* Fields Selection */}
        <div className="builder-section">
          <label>SELECT Fields:</label>
          <input
            type="text"
            value={query.select.join(", ")}
            onChange={(e) =>
              setQuery({
                ...query,
                select: e.target.value.split(",").map((s) => s.trim()),
              })
            }
            placeholder="e.g., id, title, rank_score"
          />
        </div>

        {/* WHERE Filters */}
        <div className="builder-section">
          <label>WHERE Conditions:</label>
          <div className="conditions-list">
            {query.where.map((condition, index) => (
              <input
                key={index}
                type="text"
                value={condition}
                onChange={(e) => {
                  const newWhere = [...query.where];
                  newWhere[index] = e.target.value;
                  setQuery({ ...query, where: newWhere });
                }}
              />
            ))}
            <button
              onClick={() =>
                setQuery({ ...query, where: [...query.where, ""] })
              }
            >
              + Add Condition
            </button>
          </div>
        </div>

        {/* Vector Search */}
        <div className="builder-section">
          <label>Vector Similarity Search:</label>
          <input
            type="text"
            placeholder="Vector field (e.g., embedding)"
            value={query.vectorSearch?.field || ""}
            onChange={(e) =>
              setQuery({
                ...query,
                vectorSearch: {
                  ...query.vectorSearch,
                  field: e.target.value,
                  threshold: query.vectorSearch?.threshold || 0.7,
                  query: query.vectorSearch?.query || "",
                } as any,
              })
            }
          />
          <input
            type="number"
            placeholder="Threshold (0.0 - 1.0)"
            min="0"
            max="1"
            step="0.1"
            value={query.vectorSearch?.threshold || 0.7}
            onChange={(e) =>
              setQuery({
                ...query,
                vectorSearch: {
                  ...query.vectorSearch,
                  threshold: parseFloat(e.target.value),
                } as any,
              })
            }
          />
        </div>

        {/* Order By */}
        <div className="builder-section">
          <label>ORDER BY:</label>
          <select
            value={query.orderBy[0]?.field || ""}
            onChange={(e) =>
              setQuery({
                ...query,
                orderBy: e.target.value
                  ? [{ field: e.target.value, direction: "DESC" }]
                  : [],
              })
            }
          >
            <option value="">-- Select field --</option>
            <option value="rank_score">Rank Score</option>
            <option value="published_at">Published At</option>
            <option value="views">Views</option>
          </select>
          <select
            value={query.orderBy[0]?.direction || "DESC"}
            onChange={(e) =>
              setQuery({
                ...query,
                orderBy: query.orderBy.map((o, i) =>
                  i === 0 ? { ...o, direction: e.target.value as "ASC" | "DESC" } : o
                ),
              })
            }
          >
            <option value="DESC">Descending</option>
            <option value="ASC">Ascending</option>
          </select>
        </div>

        {/* Limit */}
        <div className="builder-section">
          <label>LIMIT:</label>
          <input
            type="number"
            min="1"
            max="1000"
            value={query.limit}
            onChange={(e) =>
              setQuery({ ...query, limit: parseInt(e.target.value) })
            }
          />
        </div>
      </div>

      {/* Generated HDQL */}
      <div className="query-preview">
        <h3>Generated HDQL:</h3>
        <pre>{queryText || generateHDQL(query)}</pre>
      </div>
    </div>
  );
}
