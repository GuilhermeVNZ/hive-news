# HDQL (Hive Data Query Language) Specification

**Version:** 1.0.0  
**Last Updated:** 2025-10-26  
**Status:** Draft Specification

---

## 1. Overview

HDQL (Hive Data Query Language) is a proprietary query language designed for querying scientific content with vector similarity search, temporal filtering, and advanced aggregation capabilities.

### Design Principles

1. **Human-Readable:** Syntax that reads like natural language
2. **Type-Safe:** Compile-time query validation
3. **Vector-Aware:** Native support for semantic search
4. **Streaming:** Real-time result updates via StreamableHTTP
5. **Extensible:** Plugin system for custom functions

---

## 2. Basic Syntax

### FROM Clause

```
FROM collection_name [alias]
```

**Examples:**

```hdql
FROM articles a

FROM articles a, translations t

FROM "articles-airesearch" a
```

### WHERE Clause

```
WHERE condition [AND|OR condition]*
```

**Supported Operators:**

- Comparison: `=`, `!=`, `>`, `<`, `>=`, `<=`
- Pattern: `LIKE`, `NOT LIKE`, `IN`, `NOT IN`
- Null: `IS NULL`, `IS NOT NULL`
- Range: `BETWEEN ... AND ...`

**Examples:**

```hdql
WHERE a.rank_score > 0.7

WHERE a.published_at > '2025-01-01' AND a.status = 'published'

WHERE a.language IN ['en', 'pt', 'es']
```

### SELECT Clause

```
SELECT field [AS alias] [, field [AS alias]]*
```

**Examples:**

```hdql
SELECT a.title, a.rank_score

SELECT a.*, COUNT(t.*) AS translation_count

SELECT DISTINCT a.portal_id
```

### ORDER BY Clause

```
ORDER BY field [ASC|DESC] [, field [ASC|DESC]]*
```

**Examples:**

```hdql
ORDER BY a.rank_score DESC, a.freshness DESC

ORDER BY a.published_at ASC
```

### LIMIT Clause

```
LIMIT count [OFFSET offset]
```

**Examples:**

```hdql
LIMIT 20

LIMIT 10 OFFSET 50
```

---

## 3. Vector Similarity Operators

### Vector Similarity

```
vector_similarity(vector_field, query_vector)
vector_cosine(vector_field, query_vector)
vector_euclidean(vector_field, query_vector)
vector_dot_product(vector_field, query_vector)
```

**Examples:**

```hdql
WHERE vector_similarity(a.embedding, $query_vector) > 0.75

WHERE vector_cosine(a.embedding, [0.1, 0.2, 0.3]) > 0.8
```

### Vector Search Optimization

```hdql
# Efficient vector search with threshold
WHERE vector_similarity(a.embedding, $query)
  > 0.7  -- Minimum similarity
  AND a.portal_id = 'airesearch'
ORDER BY vector_similarity(a.embedding, $query) DESC
LIMIT 50
```

---

## 4. Full-Text Search

### SEARCH Clause

```
SEARCH "query" IN [field1, field2, ...]
```

**Examples:**

```hdql
SEARCH "artificial intelligence" IN [a.title, a.body, a.abstract]

SEARCH "machine learning breakthroughs" IN [a.title, a.body]
```

### Combined Search

```hdql
FROM articles a
WHERE a.published_at > '2025-01-01'
  AND vector_similarity(a.embedding, $query) > 0.7
SEARCH "AI research" IN [a.title, a.body]
ORDER BY a.rank_score DESC
LIMIT 20
```

---

## 5. Temporal Queries

### Date Functions

```
now()                    -- Current timestamp
date_add(date, interval) -- Add interval
date_sub(date, interval) -- Subtract interval
datediff(date1, date2)   -- Difference between dates
```

**Supported Intervals:**

- Years, months, weeks, days
- Hours, minutes, seconds

**Examples:**

```hdql
WHERE a.published_at > date_sub(now(), '7 days')

WHERE a.updated_at BETWEEN date_sub(now(), '30 days') AND now()

WHERE datediff(now(), a.published_at) < 7
```

### Temporal Filters

```hdql
# Last 7 days
WHERE a.published_at > date_sub(now(), '7 days')

# This month
WHERE YEAR(a.published_at) = YEAR(now())
  AND MONTH(a.published_at) = MONTH(now())

# Hourly aggregation
GROUP BY HOUR(a.published_at)
```

---

## 6. Aggregations

### Basic Aggregation

```
SELECT aggregate_function(field) AS alias
FROM table
GROUP BY field
```

**Aggregate Functions:**

- `COUNT(*)` - Count all rows
- `COUNT(DISTINCT field)` - Count distinct values
- `SUM(field)` - Sum of values
- `AVG(field)` - Average value
- `MIN(field)` - Minimum value
- `MAX(field)` - Maximum value

**Examples:**

```hdql
SELECT a.portal_id, COUNT(*) AS total
FROM articles a
GROUP BY a.portal_id

SELECT a.language, AVG(a.rank_score) AS avg_rank
FROM articles a
WHERE a.rank_score > 0.5
GROUP BY a.language
```

### HAVING Clause

```
HAVING aggregate_condition
```

**Example:**

```hdql
SELECT a.portal_id, COUNT(*) AS total
FROM articles a
GROUP BY a.portal_id
HAVING total > 100
```

---

## 7. Joins

### INNER JOIN

```
FROM table1 t1
INNER JOIN table2 t2 ON t1.id = t2.article_id
```

**Example:**

```hdql
FROM articles a
INNER JOIN translations t ON a.id = t.article_id
WHERE t.language = 'pt'
ORDER BY a.rank_score DESC
LIMIT 10
```

### LEFT JOIN

```
FROM table1 t1
LEFT JOIN table2 t2 ON t1.id = t2.article_id
```

### Subqueries

```
SELECT field1, (
  SELECT aggregate_function(field2)
  FROM table2
  WHERE table2.id = table1.id
) AS alias
FROM table1
```

---

## 8. Real-Time Streaming

### Stream Keyword

```
STREAM query
```

**Example:**

```hdql
STREAM
FROM articles a
WHERE a.status = 'published'
  AND a.published_at > date_sub(now(), '1 hour')
ORDER BY a.published_at DESC
```

### Stream Operations

```hdql
# Stream updates for specific criteria
STREAM
FROM articles a
WHERE a.portal_id = 'airesearch'
  AND a.rank_score > 0.8
ORDER BY a.published_at DESC
LIMIT 10

# Stream metrics
STREAM
SELECT a.portal_id, COUNT(*) AS count
FROM articles a
GROUP BY a.portal_id
```

---

## 9. Advanced Features

### Windowing Functions

```hdql
SELECT
  a.title,
  a.rank_score,
  ROW_NUMBER() OVER (PARTITION BY a.portal_id ORDER BY a.rank_score DESC) AS rank
FROM articles a
```

### Conditional Logic

```hdql
SELECT
  a.title,
  CASE
    WHEN a.rank_score > 0.8 THEN 'high'
    WHEN a.rank_score > 0.5 THEN 'medium'
    ELSE 'low'
  END AS priority
FROM articles a
```

---

## 10. Performance Optimizations

### Index Hints

```hdql
FROM articles a USE INDEX (idx_rank_score, idx_published_at)
WHERE a.rank_score > 0.7
ORDER BY a.published_at DESC
```

### Query Planning

```hdql
# Prefix EXPLAIN for query plan
EXPLAIN
FROM articles a
WHERE vector_similarity(a.embedding, $query) > 0.7
```

---

## 11. Example Queries

### Find Top Trending Articles

```hdql
FROM articles a
WHERE a.published_at > date_sub(now(), '24 hours')
  AND a.rank_score > 0.7
ORDER BY a.rank_score DESC, a.views DESC
LIMIT 10
```

### Semantic Search

```hdql
FROM articles a
WHERE vector_similarity(a.embedding, $query_vector) > 0.75
  AND a.status = 'published'
SEARCH "quantum computing" IN [a.title, a.body]
ORDER BY vector_similarity(a.embedding, $query_vector) DESC
LIMIT 20
```

### Portal Statistics

```hdql
SELECT
  a.portal_id,
  COUNT(*) AS total_articles,
  AVG(a.rank_score) AS avg_rank,
  SUM(a.views) AS total_views
FROM articles a
WHERE a.published_at > date_sub(now(), '7 days')
GROUP BY a.portal_id
HAVING total_articles > 10
ORDER BY total_views DESC
```

### Multi-Language Content

```hdql
FROM articles a
INNER JOIN translations t ON a.id = t.article_id
WHERE t.language = 'pt'
  AND a.rank_score > 0.8
ORDER BY a.published_at DESC
LIMIT 10
```

---

## 12. Implementation Details

### Query Execution

1. **Parse:** Validate syntax and types
2. **Plan:** Generate query execution plan
3. **Optimize:** Apply indexes and hints
4. **Execute:** Run against Vectorizer/Synap/PostgreSQL
5. **Stream:** Send results via StreamableHTTP

### Supported Types

- **Primitives:** `string`, `number`, `boolean`, `date`, `timestamp`
- **Complex:** `json`, `array`, `vector` (512D float array)
- **Special:** `uuid`, `uri`

---

**Authors:** Hive-News Architecture Team  
**Status:** Draft - Subject to Change
