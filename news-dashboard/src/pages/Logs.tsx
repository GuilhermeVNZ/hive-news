import { useEffect, useState } from 'react';
import axios from 'axios';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Clock, ExternalLink } from "lucide-react";

interface Destination { site_id: string; site_name: string; url: string }
interface ArticleLog { id: string; title: string; created_at: string; age_seconds: number; source: string; destinations: Destination[]; hidden: boolean; featured: boolean }

function formatRelative(sec: number) {
  if (sec < 60) return `${sec}s ago`;
  const m = Math.floor(sec/60); if (m < 60) return `${m}m ago`;
  const h = Math.floor(m/60); if (h < 24) return `${h}h ago`;
  const d = Math.floor(h/24); return `${d}d ago`;
}

export default function Logs() {
  const [items, setItems] = useState<ArticleLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [query, setQuery] = useState('');
  const [offset, setOffset] = useState(0);
  const [showFeaturedOnly, setShowFeaturedOnly] = useState(false);
  const [selectedSite, setSelectedSite] = useState<string>('all'); // Filter by site
  const [hasMore, setHasMore] = useState(true); // Track if there are more items to load
  const pageSize = 10;

  const load = async (retries = 3) => {
    try {
      setLoading(true);
      setError(''); // Limpar erro anterior
      const params: any = { 
        limit: pageSize, 
        offset,
        ...(showFeaturedOnly && { featured: true }), // Add featured filter when checkbox is checked
        ...(selectedSite && selectedSite !== 'all' && { site: selectedSite }), // Add site filter when not 'all'
      };
      if (query) params.q = query; // Add search query if present
      const res = await axios.get('/api/logs', { 
        params,
        timeout: 10000, // 10 segundos de timeout
      });
      if (res.data?.success) {
        const batch = res.data.items || [];
        // Update hasMore: if we got fewer items than pageSize, we've reached the end
        setHasMore(batch.length === pageSize);
        
        // If offset is 0, replace all items. Otherwise, append new items
        // Only append if we got new items (avoid duplicates)
        setItems(prev => {
          if (offset === 0) {
            return batch;
          }
          // Check for duplicates by ID
          const existingIds = new Set(prev.map((item: ArticleLog) => item.id));
          const newItems = batch.filter((item: ArticleLog) => !existingIds.has(item.id));
          return [...prev, ...newItems];
        });
      }
      else setError(res.data?.error || 'Failed to load logs');
    } catch (e:any) {
      // Se é erro de rede e ainda há retries, tentar novamente
      if (retries > 0 && (!e.response || e.code === 'ECONNABORTED' || e.code === 'ERR_NETWORK')) {
        await new Promise(resolve => setTimeout(resolve, 1000)); // Aguardar 1s antes de retry
        return load(retries - 1);
      }
      setError(e.response?.data?.error || e.message || 'Failed to connect to server');
    } finally { 
      setLoading(false); 
    }
  };

  // Reload when offset, query, featured filter, or site filter changes
  useEffect(()=>{ 
    load(); 
  }, [offset, query, showFeaturedOnly, selectedSite]);
  
  // Reset offset when query, featured filter, or site filter changes
  useEffect(() => {
    setOffset(0);
    setHasMore(true); // Reset hasMore when filters change
  }, [query, showFeaturedOnly, selectedSite]);

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Collection Logs</h1>
        <p className="text-muted-foreground mt-2">View and monitor collection activities</p>
      </div>

      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>Recent Articles</CardTitle>
          <CardDescription>Newest first</CardDescription>
        </CardHeader>
          <CardContent>
          <div className="mb-3 space-y-2">
            <div className="flex gap-3">
              <input
                className="flex-1 h-9 px-3 rounded-md border border-input bg-background text-sm"
                placeholder="Search by title or ID..."
                value={query}
                onChange={(e)=>setQuery(e.target.value)}
              />
              <select
                className="h-9 px-3 rounded-md border border-input bg-background text-sm cursor-pointer hover:bg-accent transition-colors"
                value={selectedSite}
                onChange={(e) => {
                  setSelectedSite(e.target.value);
                  setOffset(0); // Reset offset when filter changes
                }}
              >
                <option value="all">All Sites</option>
                <option value="airesearch">AIResearch</option>
                <option value="scienceai">ScienceAI</option>
              </select>
              <label className="flex items-center gap-2 cursor-pointer px-3 py-2 rounded-md border border-input hover:bg-accent transition-colors">
                <input
                  type="checkbox"
                  checked={showFeaturedOnly}
                  onChange={(e) => {
                    setShowFeaturedOnly(e.target.checked);
                    setOffset(0); // Reset offset when filter changes
                  }}
                  className="w-4 h-4 rounded border-gray-300 text-primary focus:ring-primary"
                />
                <span className="text-sm font-medium">Featured only</span>
              </label>
            </div>
            <div className="text-xs text-muted-foreground">
              Showing up to {pageSize} results
              {showFeaturedOnly ? ' (featured only)' : ''}
              {selectedSite !== 'all' ? ` (${selectedSite === 'airesearch' ? 'AIResearch' : 'ScienceAI'} only)` : ''}
            </div>
          </div>
          {loading ? (
            <div className="p-6 text-sm text-muted-foreground">Loading...</div>
          ) : error ? (
            <div className="p-6 text-sm text-destructive">{error}</div>
          ) : (() => {
            // Filter items based on query (featured filter is now handled by backend)
            const q = query.toLowerCase().trim();
            const tokens = q.length ? q.split(/\s+/).filter(Boolean) : [];
            let filtered = items;
            
            // Note: Featured filter is now handled by backend, so we don't need to filter here
            // But we keep it as a safety check in case backend doesn't filter correctly
            if (showFeaturedOnly) {
              filtered = filtered.filter(it => it.featured === true);
            }
            
            // Then apply search query filter
            if (tokens.length > 0) {
              filtered = filtered.filter(it => {
                // Normalize both search query and title for better matching
                // Normalize Unicode, remove extra spaces, normalize to lowercase
                const normalizeText = (text: string) => {
                  return text
                    .normalize('NFD')           // Decompose Unicode characters (é -> e + ´)
                    .replace(/[\u0300-\u036f]/g, '') // Remove diacritics (accents)
                    .toLowerCase()
                    .replace(/[^\w\s]/g, ' ')  // Replace special chars with spaces
                    .replace(/\s+/g, ' ')       // Replace multiple spaces with single space
                    .trim();
                };
                
                const normalizedTitle = normalizeText(it.title);
                const normalizedId = normalizeText(it.id);
                const normalizedQuery = normalizeText(q);
                
                // Try exact match first (normalized) - most permissive
                if (normalizedTitle === normalizedQuery || normalizedId === normalizedQuery) {
                  return true;
                }
                
                // Try substring match (normalized query must be contained in normalized title)
                // This is more flexible - allows partial matches
                if (normalizedTitle.includes(normalizedQuery) || normalizedId.includes(normalizedQuery)) {
                  return true;
                }
                
                // Reverse substring match: check if normalized title is contained in normalized query
                // This handles cases where user types more than the title
                if (normalizedQuery.includes(normalizedTitle) || normalizedQuery.includes(normalizedId)) {
                  return true;
                }
                
                // Then try token-based search (each token must appear) - more strict
                // This allows partial word matching
                const searchText = `${normalizedId} ${normalizedTitle}`;
                const normalizedTokens = tokens.map(t => normalizeText(t));
                return normalizedTokens.every(token => {
                  // Each token must appear as a whole word or substring
                  // Allow token to be at least 2 characters to avoid matching single letters
                  if (token.length < 2) return true; // Skip single character tokens
                  return searchText.includes(token);
                });
              });
            }
            
            // If no results after filtering but we have items, show message
            if (filtered.length === 0 && items.length > 0) {
              return (
                <div className="space-y-4">
                  <div className="p-6 text-sm text-muted-foreground text-center">
                    {showFeaturedOnly && query 
                      ? `No featured articles found matching "${query}"`
                      : showFeaturedOnly 
                      ? 'No featured articles found'
                      : `No articles found matching "${query}"`
                    }
                  </div>
                </div>
              );
            }
            
            // Render filtered items
            return (
              <div className="space-y-4">
                {filtered.map(item => (
                <div key={item.id} className="p-4 rounded-lg border hover:bg-accent/50 transition-colors">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-foreground">{item.title}</p>
                      <div className="flex items-center gap-3 text-sm text-muted-foreground mt-1">
                        <span className="flex items-center gap-1"><Clock size={14} /> {formatRelative(item.age_seconds)}</span>
                        <span>•</span>
                        <span>Source: <Badge variant="outline">{item.source}</Badge></span>
                        {item.hidden && (<><span>•</span><Badge variant="destructive">Hidden</Badge></>) }
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <label className="flex items-center gap-2 cursor-pointer">
                        <input
                          type="checkbox"
                          checked={item.featured}
                          onChange={async(e)=>{
                            try{
                              const newValue = e.target.checked;
                              // Otimistic update - atualizar UI imediatamente
                              setItems(prev => prev.map(it => 
                                it.id === item.id ? { ...it, featured: newValue } : it
                              ));
                              
                              const response = await axios.put(`/api/logs/articles/${item.id}/featured`, 
                                { featured: newValue },
                                { timeout: 5000 }
                              );
                              // Verify response is successful
                              if (!response.data?.success) {
                                throw new Error(response.data?.error || 'Update failed');
                              }
                              // Optimistic update already done, no need to update again
                              // Just verify it persisted correctly
                            } catch(e:any){
                              // Reverter otimistic update em caso de erro
                              setItems(prev => prev.map(it => 
                                it.id === item.id ? { ...it, featured: item.featured } : it
                              ));
                              setError(e.response?.data?.error || e.message || 'Failed to update featured status');
                            }
                          }}
                          className="w-4 h-4 rounded border-gray-300 text-primary focus:ring-primary"
                        />
                        <span className="text-sm text-muted-foreground">Featured</span>
                      </label>
                      <button
                        className={`inline-flex items-center gap-1 text-sm ${item.hidden ? 'text-green-600' : 'text-destructive'}`}
                        onClick={async()=>{
                          try{
                            const newValue = !item.hidden;
                            // Otimistic update
                            setItems(prev => prev.map(it => 
                              it.id === item.id ? { ...it, hidden: newValue } : it
                            ));
                            
                            const response = await axios.put(`/api/logs/articles/${item.id}/hidden`, 
                              { hidden: newValue },
                              { timeout: 5000 }
                            );
                            // Verify response is successful
                            if (!response.data?.success) {
                              throw new Error(response.data?.error || 'Update failed');
                            }
                            // Optimistic update already done, no need to update again
                            // Just verify it persisted correctly
                          } catch(e:any){
                            // Reverter otimistic update
                            setItems(prev => prev.map(it => 
                              it.id === item.id ? { ...it, hidden: item.hidden } : it
                            ));
                            setError(e.response?.data?.error || e.message || 'Failed to update hidden status');
                          }
                        }}
                      >
                        {item.hidden ? 'Show' : 'Hide'}
                      </button>
                    </div>
                  </div>
                  <div className="mt-2 flex gap-2 flex-wrap">
                    {item.destinations.map(d => (
                      <a key={d.site_id} href={d.url} target="_blank" className="text-xs inline-flex items-center gap-1 px-2 py-1 rounded border hover:bg-accent">
                        <ExternalLink size={12} /> {d.site_name}
                      </a>
                    ))}
                    {item.destinations.length===0 && (
                      <span className="text-xs text-muted-foreground">No destinations</span>
                    )}
                  </div>
                </div>
                ))}
                {(() => {
                  // Only show "Load more" button if:
                  // 1. We're not filtering by search query (featured filter works with pagination)
                  // 2. There are more items to load (hasMore)
                  // 3. Not currently loading
                  const showLoadMore = !query && hasMore && !loading;
                  
                  return showLoadMore ? (
                    <div className="pt-2">
                      <button
                        className="text-sm px-3 py-2 rounded border hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed"
                        onClick={()=> setOffset(prev => prev + pageSize)}
                        disabled={loading}
                    >
                      {loading ? 'Loading...' : 'Load more'}
                    </button>
                  </div>
                ) : null;
              })()}
              </div>
            );
          })()}
        </CardContent>
      </Card>

      
    </div>
  );
}
