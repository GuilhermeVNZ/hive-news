import { useEffect, useMemo, useState } from "react";
import axios from "axios";
import { Activity, Clock, CheckCircle, TrendingUp, FileCode, Globe } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type Site = { id: string; name: string; domain?: string | null; enabled?: boolean };
type LogItem = { id: string; title: string; created_at: string; age_seconds: number; destinations: { site_id: string; site_name: string; url: string }[] };

export default function Dashboard() {
  const [selectedSite, setSelectedSite] = useState<string>("");
  const [sites, setSites] = useState<Site[]>([]);
  const [logs, setLogs] = useState<LogItem[]>([]);
  const [sys, setSys] = useState<{ output_size_bytes: number; images_size_bytes: number } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        const [sitesRes, logsRes, sysRes] = await Promise.all([
          axios.get('/api/sites'),
          axios.get('/api/logs', { params: { limit: 200 } }),
          axios.get('/api/system/status')
        ]);
        if (sitesRes.data?.success) setSites(sitesRes.data.sites || sitesRes.data.pages || []);
        if (logsRes.data?.success) setLogs(logsRes.data.items || []);
        if (sysRes.data?.success) setSys({ output_size_bytes: sysRes.data.output_size_bytes, images_size_bytes: sysRes.data.images_size_bytes });
        if (!selectedSite && sitesRes.data?.sites?.length) setSelectedSite(sitesRes.data.sites[0].id);
      } catch (e:any) { setError(e.response?.data?.error || e.message); } finally { setLoading(false); }
    };
    load();
  }, []);

  const totalPages = sites.length;
  const lastPublishedAgo = useMemo(() => {
    if (logs.length === 0) return 'N/A';
    const newest = logs.reduce((a,b)=> new Date(a.created_at) > new Date(b.created_at) ? a : b);
    const sec = Math.floor((Date.now() - new Date(newest.created_at).getTime())/1000);
    if (sec < 60) return `${sec}s ago`; const m=Math.floor(sec/60); if (m<60) return `${m}m ago`; const h=Math.floor(m/60); if (h<24) return `${h}h ago`; const d=Math.floor(h/24); return `${d}d ago`;
  }, [logs]);
  const articles24h = useMemo(()=> logs.filter(l => l.age_seconds <= 24*3600).length, [logs]);

  const stats = [
    { label: "Total Pages", value: String(totalPages), icon: Activity },
    { label: "Last Published", value: lastPublishedAgo, icon: Clock },
    { label: "Articles Today", value: String(articles24h), icon: TrendingUp },
  ];

  return (
    <div className="p-8 space-y-8 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Dashboard</h1>
          <p className="text-muted-foreground mt-2">Overview of your news management system</p>
        </div>
        <div />
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat, index) => {
          const Icon = stat.icon;
          return (
            <Card key={index} className="hover-lift animate-fade-in-up border border-border">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  {stat.label}
                </CardTitle>
                <Icon className="h-5 w-5 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-foreground">{stat.value}</div>
                <div className="flex items-center gap-2 mt-2">
                  <Badge variant={stat.trend === "up" ? "default" : "outline"}>
                    {stat.trend === "up" ? (
                      <TrendingUp size={12} className="mr-1" />
                    ) : (
                      <TrendingUp size={12} className="mr-1 rotate-180" />
                    )}
                    {stat.change}
                  </Badge>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Site Status */}
      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>Site Status</CardTitle>
          <CardDescription>Online state and last 24h published</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {sites.length === 0 && (
              <p className="text-sm text-muted-foreground">No sites available</p>
            )}
            {sites.map((s)=>{
              const published24 = logs.filter(l=> l.destinations.some(d=> d.site_id.toLowerCase()===s.id.toLowerCase()) && l.age_seconds<=24*3600).length;
              const online = !!s.domain && s.domain.length>0;
              return (
                <div key={s.id} className="flex items-center justify-between p-3 rounded-lg border hover:bg-accent/50">
                  <div className="flex items-center gap-3">
                    <div className={`w-3 h-3 rounded-full ${online? 'bg-green-500':'bg-red-500'}`} />
                    <div>
                      <p className="font-medium text-foreground">{s.name}</p>
                      <p className="text-sm text-muted-foreground">{published24} articles in last 24h</p>
                    </div>
                  </div>
                  <div className="text-xs text-muted-foreground flex items-center gap-1"><Globe className="w-3 h-3"/>{s.domain || 'offline'}</div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
          <CardDescription>Start collection for a site</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <select value={selectedSite} onChange={(e)=> setSelectedSite(e.target.value)} className="border border-input bg-background rounded-md px-3 py-2 text-sm">
              {sites.map(s=> (<option key={s.id} value={s.id}>{s.name}</option>))}
            </select>
            <Button variant="default" className="justify-start" onClick={async()=>{ if(!selectedSite) return; try{ await axios.post(`/api/sites/${selectedSite}/collect/start`); } catch(e:any){ setError(e.response?.data?.error || e.message); } }}>
              <Activity size={16} className="mr-2" />
              Start Collect
            </Button>
          </CardContent>
        </Card>

        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
          <CardTitle>System Status</CardTitle>
          <CardDescription>Backend and storage</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Backend API</span>
                <span className="text-sm">Online</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Output folder size</span>
                <span className="text-sm">{sys ? (Math.round(sys.output_size_bytes/1024/1024*10)/10)+ ' MB' : '...'}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Images total size</span>
                <span className="text-sm">{sys ? (Math.round(sys.images_size_bytes/1024/1024*10)/10)+ ' MB' : '...'}</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
