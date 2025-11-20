import { useEffect, useMemo, useState } from "react";
import axios from "axios";
import { Activity, Clock, CheckCircle, TrendingUp, FileCode, Globe, Play, Square, RefreshCw, Server, Database, Zap, FileText, BarChart3 } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type Site = { id: string; name: string; domain?: string | null; enabled?: boolean };
type LogItem = { id: string; title: string; created_at: string; age_seconds: number; destinations: { site_id: string; site_name: string; url: string }[] };
type LoopStats = {
  current_cycle: number;
  articles_by_source: Record<string, number>;
  articles_written_by_site: Record<string, number>;
  tokens_total: number;
  tokens_saved: number;
  tokens_used: number;
  last_cycle_completed_at: string | null;
};
type CollectionStatus = {
  interval_minutes: number;
  filter_score_min: number;
  max_cycles: number | null;
  enabled: boolean;
  cooldown_remaining_seconds: number;
  cooldown_total_seconds: number;
};
type ServiceStatus = {
  name: string;
  url: string;
  online: boolean;
};

export default function Dashboard() {
  const [selectedSite, setSelectedSite] = useState<string>("");
  const [sites, setSites] = useState<Site[]>([]);
  const [logs, setLogs] = useState<LogItem[]>([]);
  const [sys, setSys] = useState<{ output_size_bytes: number; images_size_bytes: number } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [loopConfig, setLoopConfig] = useState({
    interval_minutes: 30,
    filter_score_min: 0.4,
    max_cycles: null as number | null,
    enabled: false
  });
  const [isLoopRunning, setIsLoopRunning] = useState(false);
  const [loopStats, setLoopStats] = useState<LoopStats | null>(null);
  const [collectionStatus, setCollectionStatus] = useState<CollectionStatus | null>(null);
  const [servicesStatus, setServicesStatus] = useState<Record<string, ServiceStatus>>({});
  const [articlesTodayCount, setArticlesTodayCount] = useState<number>(0);

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        const [sitesRes, logsRes, sysRes, configRes, statsRes, collectionRes, servicesRes, todayRes] = await Promise.all([
          axios.get('/api/sites'),
          axios.get('/api/logs', { params: { limit: 10 } }),
          axios.get('/api/system/status'),
          axios.get('/api/system/config').catch(() => null),
          axios.get('/api/system/loop/stats').catch(() => null),
          axios.get('/api/system/collection/status').catch(() => null),
          axios.get('/api/system/services/status').catch(() => null),
          axios.get('/api/system/articles/today').catch(() => null)
        ]);
        if (sitesRes.data?.success) setSites(sitesRes.data.sites || sitesRes.data.pages || []);
        if (logsRes.data?.success) setLogs(logsRes.data.items || []);
        if (sysRes.data?.success) setSys({ output_size_bytes: sysRes.data.output_size_bytes, images_size_bytes: sysRes.data.images_size_bytes });
        if (configRes?.data?.loop_config) {
          setLoopConfig(configRes.data.loop_config);
        }
        if (statsRes?.data?.success) {
          setLoopStats(statsRes.data);
        }
        if (collectionRes?.data?.success) {
          setCollectionStatus(collectionRes.data);
          // Sincronizar isLoopRunning com collectionStatus.enabled (mais confiável)
          setIsLoopRunning(collectionRes.data.enabled || false);
        } else if (configRes?.data?.loop_config) {
          // Fallback: usar config se collectionStatus não disponível
          setIsLoopRunning(configRes.data.loop_config.enabled || false);
        }
        if (servicesRes?.data?.success) {
          setServicesStatus(servicesRes.data.services || {});
        }
        if (todayRes?.data?.success) {
          // Update articles24h with the actual count from the API
          setArticlesTodayCount(todayRes.data.count || 0);
        }
        if (!selectedSite && sitesRes.data?.sites?.length) setSelectedSite(sitesRes.data.sites[0].id);
      } catch (e:any) { setError(e.response?.data?.error || e.message); } finally { setLoading(false); }
    };
    load();
    const interval = setInterval(load, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const totalPages = sites.length;
  const lastPublishedAgo = useMemo(() => {
    if (logs.length === 0) return 'N/A';
    const newest = logs.reduce((a,b)=> new Date(a.created_at) > new Date(b.created_at) ? a : b);
    const sec = Math.floor((Date.now() - new Date(newest.created_at).getTime())/1000);
    if (sec < 60) return `${sec}s ago`; const m=Math.floor(sec/60); if (m<60) return `${m}m ago`; const h=Math.floor(m/60); if (h<24) return `${h}h ago`; const d=Math.floor(h/24); return `${d}d ago`;
  }, [logs]);
  const articles24h = articlesTodayCount || 0; // Use count from API instead of logs

  // Format cooldown time
  const formatCooldown = (seconds: number) => {
    if (seconds <= 0) return 'Ready';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  // Format tokens
  const formatTokens = (tokens: number) => {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(2)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(2)}K`;
    return tokens.toString();
  };

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
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Main Grid: Loop Configuration, Collection Status, Services Status */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Loop Configuration */}
        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
            <CardTitle>Loop Configuration</CardTitle>
            <CardDescription>Configure automatic pipeline loop settings</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="interval">Interval (minutes)</Label>
              <Input
                id="interval"
                type="number"
                min="1"
                value={loopConfig.interval_minutes}
                onChange={(e) => setLoopConfig({...loopConfig, interval_minutes: parseInt(e.target.value) || 30})}
                disabled={isLoopRunning}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="filter-score">Filter Score Minimum</Label>
              <Input
                id="filter-score"
                type="number"
                min="0"
                max="1"
                step="0.01"
                value={loopConfig.filter_score_min}
                onChange={(e) => setLoopConfig({...loopConfig, filter_score_min: parseFloat(e.target.value) || 0.4})}
                disabled={isLoopRunning}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="max-cycles">Max Cycles (leave empty for infinite)</Label>
              <Input
                id="max-cycles"
                type="number"
                min="1"
                value={loopConfig.max_cycles || ''}
                onChange={(e) => setLoopConfig({...loopConfig, max_cycles: e.target.value ? parseInt(e.target.value) : null})}
                placeholder="∞"
                disabled={isLoopRunning}
              />
            </div>
            <div className="flex gap-2">
              {!isLoopRunning ? (
                <Button
                  variant="default"
                  className="flex-1"
                  onClick={async () => {
                    try {
                      const response = await axios.post('/api/system/loop/start', loopConfig);
                      setError('');
                      // Recarregar status do backend para sincronizar
                      const [configRes, collectionRes] = await Promise.all([
                        axios.get('/api/system/config').catch(() => null),
                        axios.get('/api/system/collection/status').catch(() => null)
                      ]);
                      if (configRes?.data?.loop_config) {
                        setLoopConfig(configRes.data.loop_config);
                        setIsLoopRunning(configRes.data.loop_config.enabled || false);
                      }
                      if (collectionRes?.data?.success) {
                        setCollectionStatus(collectionRes.data);
                      }
                    } catch (e: any) {
                      setError(e.response?.data?.error || e.message);
                    }
                  }}
                >
                  <Play size={16} className="mr-2" />
                  Start Pipeline
                </Button>
              ) : (
                <Button
                  variant="destructive"
                  className="flex-1"
                  onClick={async () => {
                    try {
                      await axios.post('/api/system/loop/stop');
                      setError('');
                      // Recarregar status do backend para sincronizar
                      const [configRes, collectionRes] = await Promise.all([
                        axios.get('/api/system/config').catch(() => null),
                        axios.get('/api/system/collection/status').catch(() => null)
                      ]);
                      if (configRes?.data?.loop_config) {
                        setLoopConfig(configRes.data.loop_config);
                        setIsLoopRunning(configRes.data.loop_config.enabled || false);
                      }
                      if (collectionRes?.data?.success) {
                        setCollectionStatus(collectionRes.data);
                      }
                    } catch (e: any) {
                      setError(e.response?.data?.error || e.message);
                    }
                  }}
                >
                  <Square size={16} className="mr-2" />
                  Stop Pipeline
                </Button>
              )}
            </div>
            <Button
              variant="outline"
              className="w-full"
              onClick={async () => {
                try {
                  await axios.post('/api/system/servers/refresh');
                  setError('');
                } catch (e: any) {
                  setError(e.response?.data?.error || e.message);
                }
              }}
            >
              <RefreshCw size={16} className="mr-2" />
              Refresh Servers
            </Button>
          </CardContent>
        </Card>

        {/* Collection Status */}
        <Card className="hover-lift animate-fade-in-up">
        <CardHeader>
            <CardTitle>Collection Status</CardTitle>
            <CardDescription>Current collection cycle and cooldown</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Loop Status</span>
                <Badge variant={collectionStatus?.enabled ? "default" : "outline"}>
                  {collectionStatus?.enabled ? "Running" : "Stopped"}
                </Badge>
              </div>
              {collectionStatus && (
                <>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Current Cycle</span>
                    <span className="text-sm font-semibold">{loopStats?.current_cycle || 0}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Interval</span>
                    <span className="text-sm">{collectionStatus.interval_minutes} min</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Filter Score</span>
                    <span className="text-sm">{collectionStatus.filter_score_min.toFixed(2)}</span>
                  </div>
                  <div className="pt-2 border-t">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium">Cooldown</span>
                      <span className={`text-sm font-semibold ${collectionStatus.cooldown_remaining_seconds > 0 ? 'text-orange-600' : 'text-green-600'}`}>
                        {formatCooldown(collectionStatus.cooldown_remaining_seconds)}
                      </span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full ${collectionStatus.cooldown_remaining_seconds > 0 ? 'bg-orange-500' : 'bg-green-500'}`}
                        style={{
                          width: `${Math.max(0, Math.min(100, (collectionStatus.cooldown_remaining_seconds / collectionStatus.cooldown_total_seconds) * 100))}%`
                        }}
                      />
                    </div>
                  </div>
                </>
              )}
          </div>
        </CardContent>
      </Card>

        {/* Services Status */}
        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
            <CardTitle>Services Status</CardTitle>
            <CardDescription>Backend and frontend services</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {Object.entries(servicesStatus).map(([key, service]) => (
                <div key={key} className="flex items-center justify-between p-2 rounded-lg border hover:bg-accent/50">
                  <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${service.online ? 'bg-green-500' : 'bg-red-500'}`} />
                    <div>
                      <p className="text-sm font-medium">{service.name}</p>
                      <p className="text-xs text-muted-foreground">{service.url}</p>
                    </div>
                  </div>
                  <Badge variant={service.online ? "default" : "destructive"}>
                    {service.online ? "Online" : "Offline"}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Loop Statistics */}
      {loopStats && (
        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5" />
              Loop Statistics
            </CardTitle>
            <CardDescription>Detailed statistics from the last cycle</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
              {/* Tokens Stats */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Total Tokens</span>
                  <span className="text-sm font-semibold">{formatTokens(loopStats.tokens_total)}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Tokens Saved</span>
                  <span className="text-sm font-semibold text-green-600">{formatTokens(loopStats.tokens_saved)}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Tokens Used</span>
                  <span className="text-sm font-semibold text-blue-600">{formatTokens(loopStats.tokens_used)}</span>
                </div>
                {loopStats.tokens_total > 0 && (
                  <div className="pt-2 border-t">
                    <div className="text-xs text-muted-foreground">
                      Savings: {((loopStats.tokens_saved / loopStats.tokens_total) * 100).toFixed(1)}%
                    </div>
                  </div>
                )}
              </div>

              {/* Articles by Source */}
              <div className="space-y-2">
                <h4 className="text-sm font-semibold mb-2">Articles by Source</h4>
                <div className="space-y-1 max-h-48 overflow-y-auto">
                  {Object.entries(loopStats.articles_by_source).length > 0 ? (
                    Object.entries(loopStats.articles_by_source).map(([source, count]) => (
                      <div key={source} className="flex items-center justify-between text-xs">
                        <span className="text-muted-foreground truncate">{source}</span>
                        <span className="font-semibold ml-2">{count}</span>
                      </div>
                    ))
                  ) : (
                    <p className="text-xs text-muted-foreground">No data available</p>
                  )}
                </div>
              </div>

              {/* Articles Written by Site */}
              <div className="space-y-2">
                <h4 className="text-sm font-semibold mb-2">Articles Written by Site</h4>
                <div className="space-y-1 max-h-48 overflow-y-auto">
                  {Object.entries(loopStats.articles_written_by_site).length > 0 ? (
                    Object.entries(loopStats.articles_written_by_site).map(([site, count]) => (
                      <div key={site} className="flex items-center justify-between text-xs">
                        <span className="text-muted-foreground truncate">{site}</span>
                        <span className="font-semibold ml-2">{count}</span>
                      </div>
                    ))
                  ) : (
                    <p className="text-xs text-muted-foreground">No data available</p>
                  )}
                </div>
              </div>

              {/* Cycle Info */}
              <div className="space-y-2">
              <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Current Cycle</span>
                  <span className="text-sm font-semibold">{loopStats.current_cycle}</span>
                </div>
                {loopStats.last_cycle_completed_at && (
                  <div className="pt-2 border-t">
                    <div className="text-xs text-muted-foreground">Last Completed</div>
                    <div className="text-xs font-medium">
                      {new Date(loopStats.last_cycle_completed_at).toLocaleString()}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {error && (
        <Card className="border-red-500 bg-red-50 dark:bg-red-950">
          <CardContent className="pt-6">
            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
