# News Management System - Architecture

## Overview

The News Management System is a multi-tier architecture designed to manage content aggregation, curation, and distribution across multiple portals.

## Architecture Layers

### 1. Backend Layer (Rust + Axum + PostgreSQL)

**Location**: `news-backend/`

**Components**:
- **HTTP Server** (Axum): RESTful API endpoints
- **Database Layer** (SQLx): PostgreSQL interactions
- **Authentication** (JWT + bcrypt): Secure user sessions
- **Services**: Business logic layer

**Key Features**:
- RESTful API for dashboard operations
- JWT-based authentication
- PostgreSQL for persistent storage
- Structured logging with tracing

### 2. Dashboard Frontend (React + Tauri)

**Location**: `news-dashboard/`

**Components**:
- **React UI**: Component-based interface
- **Tauri Shell**: Desktop application wrapper
- **TanStack Query**: Server state management
- **Tailwind CSS**: Styling

**Key Features**:
- Real-time status updates
- Configuration management UI
- Cross-platform desktop app (Windows, macOS, Linux)

### 3. Content Portals (Next.js)

**Location**: `News-main/apps/frontend-next/`

**Components**:
- **AIResearch**: Scientific news portal
- **ScienceAI**: Technical research portal (future)

## Database Schema

### pages_config
Stores configuration for each content portal.

```sql
CREATE TABLE pages_config (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    sources TEXT[] DEFAULT '{}',
    frequency_minutes INT NOT NULL DEFAULT 60,
    writing_style TEXT NOT NULL DEFAULT 'scientific',
    linked_accounts JSONB DEFAULT '{}',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### users
Stores user authentication information.

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### collection_logs
Tracks automated collection runs.

```sql
CREATE TABLE collection_logs (
    id SERIAL PRIMARY KEY,
    page_id INT REFERENCES pages_config(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    articles_collected INT DEFAULT 0,
    duration_ms INT,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## API Endpoints

### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user

### Pages
- `GET /api/pages` - List all pages
- `POST /api/pages` - Create new page
- `GET /api/pages/:id` - Get page details
- `PUT /api/pages/:id` - Update page
- `DELETE /api/pages/:id` - Delete page

### Sources
- `GET /api/sources` - List sources
- `POST /api/sources` - Create source

### Logs
- `GET /api/logs` - Get collection logs

## Development Workflow

1. **Setup Database**:
   ```bash
   createdb news_system
   psql news_system -f news-backend/migrations/001_create_tables.sql
   ```

2. **Run Backend**:
   ```bash
   cd news-backend
   cargo run
   ```

3. **Run Frontend**:
   ```bash
   cd news-dashboard
   npm install
   npm run dev
   ```

## Technology Stack

### Backend
- **Language**: Rust (Edition 2024)
- **Framework**: Axum 0.7
- **Database**: PostgreSQL
- **Authentication**: JWT + bcrypt
- **Async**: Tokio

### Frontend
- **Framework**: React 18
- **Desktop**: Tauri 1.5
- **State**: TanStack Query
- **Styling**: Tailwind CSS

### Future Extensions
- WebSocket for real-time updates
- Social media integration (LinkedIn, X, YouTube)
- Automated content collection workers
- Article ranking and curation system


