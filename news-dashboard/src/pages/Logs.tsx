import { useEffect, useState } from 'react';
import axios from 'axios';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Clock, ExternalLink } from "lucide-react";

interface Destination { site_id: string; site_name: string; url: string }
interface ArticleLog { id: string; title: string; created_at: string; age_seconds: number; source: string; destinations: Destination[]; hidden: boolean }

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
  const pageSize = 10;

  const load = async () => {
    try {
      setLoading(true);
      const params: any = { limit: pageSize, offset };
      const res = await axios.get('/api/logs', { params });
      if (res.data?.success) {
        const batch = res.data.items || [];
        setItems(prev => offset === 0 ? batch : [...prev, ...batch]);
      }
      else setError(res.data?.error || 'Failed to load logs');
    } catch (e:any) { setError(e.response?.data?.error || e.message); } finally { setLoading(false); }
  };

  useEffect(()=>{ load(); }, [offset]);

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
          <div className="mb-3">
            <input
              className="w-full h-9 px-3 rounded-md border border-input bg-background text-sm"
              placeholder="Search by title or ID..."
              value={query}
              onChange={(e)=>setQuery(e.target.value)}
            />
            <div className="text-xs text-muted-foreground mt-1">Showing up to 10 results</div>
          </div>
          {loading ? (
            <div className="p-6 text-sm text-muted-foreground">Loading...</div>
          ) : error ? (
            <div className="p-6 text-sm text-destructive">{error}</div>
          ) : (
            <div className="space-y-4">
              {(() => {
                const q = query.toLowerCase().trim();
                const tokens = q.length ? q.split(/\s+/).filter(Boolean) : [];
                const base = items.slice().reverse();
                const filtered = tokens.length === 0 ? base : base.filter(it => {
                  const hay = `${it.id} ${it.title}`.toLowerCase();
                  return tokens.every(t => hay.includes(t));
                });
                return filtered;
              })().map(item => (
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
                    <div className="flex items-center gap-2">
                      <button
                        className={`inline-flex items-center gap-1 text-sm ${item.hidden ? 'text-green-600' : 'text-destructive'}`}
                        onClick={async()=>{
                          try{
                            await axios.put(`/api/logs/articles/${item.id}/hidden`, { hidden: !item.hidden });
                            await load();
                          } catch(e:any){ setError(e.response?.data?.error || e.message); }
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
              <div className="pt-2">
                <button
                  className="text-sm px-3 py-2 rounded border hover:bg-accent"
                  onClick={()=> setOffset(prev => prev + pageSize)}
                  disabled={loading}
                >
                  Load more
                </button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      
    </div>
  );
}
