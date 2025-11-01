import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Plus, Settings, Power, PowerOff, FileText, Globe, Globe2 } from 'lucide-react';
import axios from 'axios';

interface Collector {
  id: string;
  name: string;
  enabled: boolean;
  api_key: string | null;
  config: any;
  assigned_sites?: Array<{ id: string; name: string }>;
}

type CollectorType = 'api' | 'news_portal' | null;

interface SiteItem {
  id: string;
  name: string;
}

export default function Sources() {
  const [collectors, setCollectors] = useState<Collector[]>([]);
  const [sites, setSites] = useState<SiteItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [updating, setUpdating] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [collectorType, setCollectorType] = useState<CollectorType>(null);
  const [editingSites, setEditingSites] = useState<string | null>(null);
  
  // Form states for API collector
  const [apiForm, setApiForm] = useState({
    id: '',
    name: '',
    api_key: '',
    base_url: '',
    enabled: true,
  });
  
  // Form states for News Portal collector
  const [portalForm, setPortalForm] = useState({
    id: '',
    name: '',
    base_url: '',
    rss_feed: '',
    enabled: true,
  });

  useEffect(() => {
    loadCollectors();
    loadSites();
  }, []);

  const loadSites = async () => {
    try {
      const response = await axios.get('/api/sites');
      if (response.data.success) {
        const raw = response.data.sites || [];
        const mapped: SiteItem[] = raw.map((s: any) => ({ id: s.id, name: s.name }));
        setSites(mapped);
      }
    } catch (err: any) {
      console.error('Failed to load sites:', err);
    }
  };

  const loadCollectors = async () => {
    try {
      setLoading(true);
      const response = await axios.get('/api/collectors');
      
      if (response.data.success) {
        setCollectors(response.data.collectors || []);
      } else {
        setError(response.data.error || 'Failed to load collectors');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to load collectors');
    } finally {
      setLoading(false);
    }
  };

  const toggleCollector = async (collectorId: string, currentStatus: boolean) => {
    try {
      setUpdating(collectorId);
      const response = await axios.put(
        `/api/collectors/${collectorId}/status`,
        { enabled: !currentStatus }
      );

      if (response.data.success) {
        setCollectors(prev =>
          prev.map(c =>
            c.id === collectorId ? { ...c, enabled: !currentStatus } : c
          )
        );
      } else {
        setError(response.data.error || 'Failed to update collector');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update collector');
    } finally {
      setUpdating(null);
    }
  };

  const handleUpdateCollectorSites = async (collectorId: string, selectedSiteIds: string[]) => {
    try {
      setUpdating(collectorId);
      const response = await axios.put(`/api/collectors/${collectorId}/sites`, {
        site_ids: selectedSiteIds,
      });

      if (response.data.success) {
        // Reload collectors to get updated assigned_sites
        await loadCollectors();
        setEditingSites(null);
      } else {
        setError(response.data.error || 'Failed to update collector sites');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update collector sites');
    } finally {
      setUpdating(null);
    }
  };

  const toggleSiteAssignment = async (collectorId: string, siteId: string) => {
    const collector = collectors.find(c => c.id === collectorId);
    if (!collector) return;

    const currentSiteIds = collector.assigned_sites?.map(s => s.id) || [];
    const isAssigned = currentSiteIds.includes(siteId);

    try {
      setUpdating(collectorId);
      
      // Use the collector sites endpoint which properly handles adding/removing collectors from sites
      const newSiteIds = !isAssigned
        ? [...currentSiteIds, siteId]
        : currentSiteIds.filter(id => id !== siteId);
      
      const resp = await axios.put(`/api/collectors/${collectorId}/sites`, {
        site_ids: newSiteIds,
      });

      if (resp.data?.success) {
        // Reload collectors to get updated assigned_sites
        await loadCollectors();
        setError("");
      } else {
        setError(resp.data?.error || 'Failed to update site assignment');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update site assignment');
    } finally {
      setUpdating(null);
    }
  };

  const handleAddCollector = () => {
    setCollectorType(null);
    setApiForm({ id: '', name: '', api_key: '', base_url: '', enabled: true });
    setPortalForm({ id: '', name: '', base_url: '', rss_feed: '', enabled: true });
    setDialogOpen(true);
  };

  const handleSubmitAPI = async () => {
    try {
      // TODO: Implement backend endpoint for adding new collectors
      // For now, just show success message (backend to be implemented)
      console.log('Submitting API collector:', apiForm);
      setError('');
      
      // Placeholder - will be replaced with actual API call
      // const response = await axios.post('/api/collectors', {
      //   type: 'api',
      //   ...apiForm,
      //   config: {
      //     base_url: apiForm.base_url,
      //   },
      // });
      
      setDialogOpen(false);
      // loadCollectors(); // Reload after adding
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to add collector');
    }
  };

  const handleSubmitPortal = async () => {
    try {
      // TODO: Implement backend endpoint for adding news portals
      // For now, just show success message (backend to be implemented)
      console.log('Submitting News Portal collector:', portalForm);
      setError('');
      
      // Placeholder - will be replaced with actual API call
      // const response = await axios.post('/api/collectors', {
      //   type: 'news_portal',
      //   ...portalForm,
      //   config: {
      //     base_url: portalForm.base_url,
      //     rss_feed: portalForm.rss_feed,
      //   },
      // });
      
      setDialogOpen(false);
      // loadCollectors(); // Reload after adding
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to add collector');
    }
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Collectors</h1>
          <p className="text-muted-foreground mt-2">
            Manage article collectors and APIs
          </p>
        </div>
        <Button variant="default" className="gap-2" onClick={handleAddCollector}>
          <Plus size={20} />
          Add Collector
        </Button>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {collectors.map((collector) => (
          <Card
            key={collector.id}
            className="hover-lift animate-fade-in-up transition-all"
          >
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2">
                  {collector.name}
                  <Badge
                    variant={collector.enabled ? 'default' : 'outline'}
                    className={
                      collector.enabled
                        ? 'bg-green-500/10 text-green-700 dark:text-green-400'
                        : ''
                    }
                  >
                    {collector.enabled ? 'Active' : 'Inactive'}
                  </Badge>
                </CardTitle>
              </div>
              <CardDescription>
                ID: <code className="text-xs bg-muted px-1 rounded">{collector.id}</code>
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-col gap-2">
                {/* APIs públicas não precisam de API key */}
                {['arxiv', 'pmc'].includes(collector.id.toLowerCase()) ? (
                  <div className="text-sm text-muted-foreground">
                    API Key: <span className="text-green-600 dark:text-green-400">✓ Public API (not required)</span>
                  </div>
                ) : collector.id.toLowerCase() === 'semantic_scholar' ? (
                  collector.api_key ? (
                    <div className="text-sm text-muted-foreground">
                      API Key: <span className="text-green-600 dark:text-green-400">✓ Configured (optional, increases rate limit)</span>
                    </div>
                  ) : (
                    <div className="text-sm text-muted-foreground">
                      API Key: <span className="text-blue-600 dark:text-blue-400">○ Public API (optional key available)</span>
                    </div>
                  )
                ) : collector.api_key ? (
                  <div className="text-sm text-muted-foreground">
                    API Key: <span className="text-green-600 dark:text-green-400">✓ Configured</span>
                  </div>
                ) : (
                  <div className="text-sm text-muted-foreground">
                    API Key: <span className="text-orange-600 dark:text-orange-400">✗ Not set</span>
                  </div>
                )}
              </div>

              {/* Sites Assignment */}
              <div className="border-t pt-4 mt-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Globe2 className="w-4 h-4 text-primary" />
                    <p className="text-sm font-semibold">Apply to Sites</p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setEditingSites(editingSites === collector.id ? null : collector.id)}
                    disabled={updating === collector.id}
                    className="h-7 px-3 text-xs"
                  >
                    {editingSites === collector.id ? 'Cancel' : 'Select Sites'}
                  </Button>
                </div>
                {editingSites === collector.id ? (
                  <div className="space-y-2 bg-muted/30 p-3 rounded-lg border border-border">
                    <p className="text-xs text-muted-foreground mb-2">Select which sites should use this source:</p>
                    {sites.map((site) => {
                      const isAssigned = collector.assigned_sites?.some(s => s.id === site.id) || false;
                      return (
                        <label
                          key={site.id}
                          className="flex items-center gap-3 p-2 rounded-md hover:bg-background cursor-pointer transition-colors border border-transparent hover:border-primary/20"
                        >
                          <input
                            type="checkbox"
                            checked={isAssigned}
                            onChange={() => toggleSiteAssignment(collector.id, site.id)}
                            disabled={updating === collector.id}
                            className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                          />
                          <span className="text-sm font-medium">{site.name}</span>
                          {updating === collector.id && (
                            <span className="text-xs text-muted-foreground ml-auto">Updating...</span>
                          )}
                        </label>
                      );
                    })}
                    {sites.length === 0 && (
                      <p className="text-xs text-muted-foreground italic text-center py-2">No sites available. Add a site in the Sites tab.</p>
                    )}
                  </div>
                ) : (
                  <div className="flex gap-2 flex-wrap min-h-[32px] items-center">
                    {collector.assigned_sites && collector.assigned_sites.length > 0 ? (
                      collector.assigned_sites.map((site, idx) => (
                        <Badge 
                          key={idx} 
                          variant="secondary" 
                          className="text-xs bg-primary/10 text-primary border-primary/20"
                        >
                          <Globe2 className="w-3 h-3 mr-1" />
                          {site.name}
                        </Badge>
                      ))
                    ) : (
                      <span className="text-xs text-muted-foreground italic">
                        No sites assigned - this source won't collect articles for any site
                      </span>
                    )}
                  </div>
                )}
              </div>

              <div className="flex gap-2">
                <Button
                  variant={collector.enabled ? 'destructive' : 'default'}
                  className="flex-1"
                  onClick={() => toggleCollector(collector.id, collector.enabled)}
                  disabled={updating === collector.id}
                >
                  {updating === collector.id ? (
                    'Updating...'
                  ) : collector.enabled ? (
                    <>
                      <PowerOff className="w-4 h-4 mr-2" />
                      Disable
                    </>
                  ) : (
                    <>
                      <Power className="w-4 h-4 mr-2" />
                      Enable
                    </>
                  )}
                </Button>
                <Button variant="outline" size="icon" title="Configure">
                  <Settings className="w-4 h-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {collectors.length === 0 && !loading && (
        <div className="text-center py-12 text-muted-foreground">
          <p>No collectors configured yet.</p>
          <p className="text-sm mt-2">Click "Add Collector" to add a new one.</p>
        </div>
      )}

      {/* Add Collector Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent onClose={() => setDialogOpen(false)} className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Add New Collector</DialogTitle>
            <DialogDescription>
              Choose the type of collector you want to add
            </DialogDescription>
          </DialogHeader>

          {!collectorType ? (
            <div className="space-y-4 py-4">
              <div className="grid grid-cols-2 gap-4">
                <button
                  onClick={() => setCollectorType('api')}
                  className="p-6 border-2 border-border rounded-lg hover:border-primary hover:bg-primary/5 transition-all text-left"
                >
                  <FileText className="h-8 w-8 mb-2 text-primary" />
                  <h3 className="font-semibold mb-1">Article API</h3>
                  <p className="text-sm text-muted-foreground">
                    Add an API for fetching academic articles and papers
                  </p>
                </button>

                <button
                  onClick={() => setCollectorType('news_portal')}
                  className="p-6 border-2 border-border rounded-lg hover:border-primary hover:bg-primary/5 transition-all text-left"
                >
                  <Globe className="h-8 w-8 mb-2 text-primary" />
                  <h3 className="font-semibold mb-1">News Portal</h3>
                  <p className="text-sm text-muted-foreground">
                    Add a news portal or RSS feed source
                  </p>
                </button>
              </div>
            </div>
          ) : collectorType === 'api' ? (
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="api-id">Collector ID *</Label>
                <Input
                  id="api-id"
                  placeholder="e.g., custom_api"
                  value={apiForm.id}
                  onChange={(e) => setApiForm({ ...apiForm, id: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  Unique identifier (lowercase, no spaces)
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="api-name">Name *</Label>
                <Input
                  id="api-name"
                  placeholder="e.g., Custom Academic API"
                  value={apiForm.name}
                  onChange={(e) => setApiForm({ ...apiForm, name: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="api-base-url">Base URL *</Label>
                <Input
                  id="api-base-url"
                  type="url"
                  placeholder="https://api.example.com"
                  value={apiForm.base_url}
                  onChange={(e) => setApiForm({ ...apiForm, base_url: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="api-key">API Key</Label>
                <Input
                  id="api-key"
                  type="password"
                  placeholder="Leave empty if not required"
                  value={apiForm.api_key}
                  onChange={(e) => setApiForm({ ...apiForm, api_key: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  Optional - some APIs don't require authentication
                </p>
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => setCollectorType(null)}>
                  Back
                </Button>
                <Button onClick={handleSubmitAPI} disabled={!apiForm.id || !apiForm.name || !apiForm.base_url}>
                  Add API Collector
                </Button>
              </DialogFooter>
            </div>
          ) : (
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="portal-id">Portal ID *</Label>
                <Input
                  id="portal-id"
                  placeholder="e.g., techcrunch"
                  value={portalForm.id}
                  onChange={(e) => setPortalForm({ ...portalForm, id: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  Unique identifier (lowercase, no spaces)
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="portal-name">Portal Name *</Label>
                <Input
                  id="portal-name"
                  placeholder="e.g., TechCrunch"
                  value={portalForm.name}
                  onChange={(e) => setPortalForm({ ...portalForm, name: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="portal-base-url">Base URL *</Label>
                <Input
                  id="portal-base-url"
                  type="url"
                  placeholder="https://techcrunch.com"
                  value={portalForm.base_url}
                  onChange={(e) => setPortalForm({ ...portalForm, base_url: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="portal-rss">RSS Feed URL</Label>
                <Input
                  id="portal-rss"
                  type="url"
                  placeholder="https://techcrunch.com/feed/"
                  value={portalForm.rss_feed}
                  onChange={(e) => setPortalForm({ ...portalForm, rss_feed: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  Optional - RSS feed URL for news collection
                </p>
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => setCollectorType(null)}>
                  Back
                </Button>
                <Button onClick={handleSubmitPortal} disabled={!portalForm.id || !portalForm.name || !portalForm.base_url}>
                  Add News Portal
                </Button>
              </DialogFooter>
            </div>
          )}

          {error && (
            <div className="mt-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
              {error}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
