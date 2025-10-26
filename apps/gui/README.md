# Hive-News GUI

Electron-based desktop application for Hive-News with HDQL visual query builder.

## Features

- 🎯 **Visual HDQL Query Builder** - Create complex queries with a user-friendly interface
- 📊 **Real-time Results Viewer** - View and explore query results
- 🔍 **Semantic Search** - Vector similarity search support
- 📈 **Metrics Dashboard** - Real-time metrics and analytics
- ⚙️ **Portal Management** - Configure and manage news portals

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for production
npm run build

# Run built application
npm start
```

## Architecture

```
gui/
├── src/
│   ├── main/           # Electron main process
│   │   └── main.ts     # Window management, IPC handlers
│   ├── preload/        # Preload scripts (bridge)
│   │   └── preload.ts  # Safe API exposure
│   └── renderer/       # React UI
│       ├── App.tsx     # Main application
│       ├── components/ # React components
│       │   ├── HDQLQueryBuilder.tsx
│       │   └── ResultsViewer.tsx
│       └── styles/     # CSS styles
└── package.json
```

## HDQL Query Builder

The GUI provides a visual interface for building HDQL queries:

### Supported Operations

- **FROM** - Select collections/data sources
- **SELECT** - Choose fields to retrieve
- **WHERE** - Filter conditions
- **Vector Search** - Semantic similarity search
- **ORDER BY** - Sort results
- **LIMIT** - Result pagination

### Example Queries

```hdql
FROM articles a
WHERE a.published_at > date_sub(now(), '7 days')
  AND a.rank_score > 0.7
ORDER BY a.rank_score DESC, a.published_at DESC
LIMIT 20
```

## Integration

The GUI communicates with the backend via:
- **MCP Protocol** - Model Context Protocol for AI assistance
- **HTTP API** - REST endpoints for data operations
- **WebSocket** - Real-time updates

## License

MIT
