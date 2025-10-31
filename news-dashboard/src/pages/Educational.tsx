import { useEffect, useState } from 'react';
import axios from 'axios';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Globe2 } from 'lucide-react';

interface SiteItem { id: string; name: string; writer?: { provider?: string; enabled?: boolean; api_key?: string | null }; education_sources?: Array<{ id: string; enabled: boolean; api_key?: string | null }>; }

type Provider = { id: string; name: string };

const DEFAULT_EDU_PROVIDERS: Provider[] = [
  { id: 'edx', name: 'edX' },
  { id: 'mit_ocw', name: 'MIT OpenCourseWare' },
  { id: 'class_central', name: 'Class Central' },
];

export default function Educational() {
  const [sites, setSites] = useState<SiteItem[]>([]);
  const [providers, setProviders] = useState<Provider[]>(DEFAULT_EDU_PROVIDERS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [updating, setUpdating] = useState<string | null>(null);
  const [editingSites, setEditingSites] = useState<string | null>(null);
  const [apiKeyByProvider, setApiKeyByProvider] = useState<Record<string, string>>({});
  const [selectedSitesByProvider, setSelectedSitesByProvider] = useState<Record<string, string[]>>({});
  const [addOpen, setAddOpen] = useState(false);
  const [newProv, setNewProv] = useState({ id: '', name: '' });

  useEffect(() => {
    loadSites();
    // Load custom providers from localStorage
    try {
      const raw = localStorage.getItem('education_providers');
      if (raw) {
        const extra = JSON.parse(raw) as Provider[];
        if (Array.isArray(extra) && extra.length > 0) setProviders([...DEFAULT_EDU_PROVIDERS, ...extra]);
      }
    } catch {}
  }, []);

  const addProvider = () => {
    const id = newProv.id.trim().toLowerCase();
    const name = newProv.name.trim();
    if (!id || !name) { setError('Provider ID and Name are required'); return; }
    if (providers.some(p => p.id === id)) { setError('Provider ID already exists'); return; }
    const extra = providers.filter(p => !DEFAULT_EDU_PROVIDERS.find(d => d.id === p.id));
    const updated = [...DEFAULT_EDU_PROVIDERS, ...extra, { id, name }];
    setProviders(updated);
    try { localStorage.setItem('education_providers', JSON.stringify(updated.filter(p => !DEFAULT_EDU_PROVIDERS.find(d => d.id === p.id)))); } catch {}
    setAddOpen(false);
    setNewProv({ id: '', name: '' });
    setError('');
  };

  const loadSites = async () => {
    try {
      setLoading(true);
      const resp = await axios.get('/api/sites');
      if (resp.data?.success) {
        setSites(resp.data.sites || []);
        const keys: Record<string, string> = {};
        (resp.data.sites || []).forEach((s: any) => {
          (s.education_sources || []).forEach((src: any) => {
            if (src.api_key && !keys[src.id]) keys[src.id] = src.api_key;
          });
        });
        setApiKeyByProvider(keys);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to load sites');
    } finally { setLoading(false); }
  };

  const assignedSites = (providerId: string) => {
    return sites
      .filter(s =>
        // Primary: education_sources
        (s.education_sources || []).some(src => src.id === providerId && src.enabled)
        // Fallback: if backend not updated, reuse writer provider state
        || (s as any).writer?.provider === providerId && (s as any).writer?.enabled !== false
      )
      .map(s => ({ id: s.id, name: s.name }));
  };

  const toggleProviderForSite = async (providerId: string, siteId: string, enabled: boolean) => {
    try {
      setUpdating(`${providerId}:${siteId}`);
      // 1) Update status (enable/disable)
      try {
        await axios.put(`/api/sites/${siteId}/education/${providerId}/status`, { enabled });
      } catch (err: any) {
        // Fallback for older backends
        await axios.put(`/api/sites/${siteId}/writer`, { provider: providerId, enabled });
      }
      // 2) If API key present, update config as well
      if (apiKeyByProvider[providerId]) {
        try {
          await axios.put(`/api/sites/${siteId}/education/${providerId}/config`, {
            api_key: apiKeyByProvider[providerId],
            enabled,
          });
        } catch (err: any) {
          await axios.put(`/api/sites/${siteId}/writer`, { provider: providerId, enabled, api_key: apiKeyByProvider[providerId] });
        }
      }
      await loadSites();
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update provider');
    } finally { setUpdating(null); }
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-8">
      {error && <div className="p-3 bg-destructive/10 border border-destructive/20 rounded text-destructive">{error}</div>}

      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">Education</h2>
            <p className="text-muted-foreground">Select which Education API to use per article</p>
          </div>
          <Button variant="default" className="gap-2" onClick={() => { setNewProv({ id: '', name: '' }); setAddOpen(true); }}>
            Add New
          </Button>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {providers.map((prov) => {
            const assigned = assignedSites(prov.id);
            const active = assigned.length > 0;
            return (
              <Card key={prov.id} className="hover-lift animate-fade-in-up">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">
                      {prov.name}
                      <span className={`text-xs px-2 py-0.5 rounded ${active ? 'bg-green-500/10 text-green-700 dark:text-green-400' : 'border border-border text-muted-foreground'}`}>
                        {active ? 'Active' : 'Inactive'}
                      </span>
                    </CardTitle>
                  </div>
                  <CardDescription>ID: <code className="text-xs bg-muted px-1 rounded">{prov.id}</code></CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="text-sm text-muted-foreground">
                    API Key{' '}
                    {apiKeyByProvider[prov.id] ? (
                      <span className="text-green-600 dark:text-green-400">✓ Configured</span>
                    ) : (
                      <span className="text-orange-600 dark:text-orange-400">✗ Not set</span>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <input
                      className="flex-1 h-9 px-3 rounded-md border border-input bg-background text-sm"
                      type="password"
                      placeholder="Enter API key"
                      value={apiKeyByProvider[prov.id] || ''}
                      onChange={(e) => setApiKeyByProvider({ ...apiKeyByProvider, [prov.id]: e.target.value })}
                    />
                    <Button
                      variant={active ? 'destructive' : 'default'}
                      onClick={async () => {
                        const targets = selectedSitesByProvider[prov.id] || [];
                        for (const siteId of targets) {
                          await toggleProviderForSite(prov.id, siteId, !active);
                        }
                      }}
                      disabled={updating !== null}
                    >
                      {active ? 'Disable' : 'Enable'}
                    </Button>
                  </div>
                  <div className="border-t pt-3">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Globe2 className="w-4 h-4 text-primary" />
                        <p className="text-sm font-semibold">Apply to Sites</p>
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setEditingSites(editingSites === prov.id ? null : prov.id)}
                        className="h-7 px-3 text-xs"
                        disabled={updating !== null}
                      >
                        {editingSites === prov.id ? 'Cancel' : 'Select Sites'}
                      </Button>
                    </div>
                    {editingSites === prov.id ? (
                      <div className="space-y-2 bg-muted/30 p-3 rounded-lg border border-border">
                        {sites.map((s) => {
                          const isAssigned = (s.education_sources || []).some(src => src.id === prov.id && src.enabled) || ((s as any).writer?.provider === prov.id && (s as any).writer?.enabled !== false);
                          return (
                            <label key={s.id} className="flex items-center gap-3 p-2 rounded-md hover:bg-background cursor-pointer transition-colors border border-transparent hover:border-primary/20">
                              <input
                                type="checkbox"
                                checked={isAssigned}
                                onChange={() => toggleProviderForSite(prov.id, s.id, !isAssigned)}
                                className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                              />
                              <span className="text-sm font-medium">{s.name}</span>
                            </label>
                          );
                        })}
                        {sites.length === 0 && (
                          <p className="text-xs text-muted-foreground italic text-center py-2">No sites available</p>
                        )}
                      </div>
                    ) : null}
                  </div>
                  <div className="flex gap-2 flex-wrap mt-2">
                    {assigned.length > 0 ? assigned.map((site, idx) => (
                      <Badge key={idx} variant="secondary" className="text-xs bg-primary/10 text-primary border-primary/20">
                        <Globe2 className="w-3 h-3 mr-1" />{site.name}
                      </Badge>
                    )) : (
                      <span className="text-xs text-muted-foreground italic">No sites assigned</span>
                    )}
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </section>
      <Dialog open={addOpen} onOpenChange={setAddOpen}>
        <DialogContent onClose={() => setAddOpen(false)} className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Add Education Provider</DialogTitle>
            <DialogDescription>Register a new education API to use in your sites.</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label htmlFor="prov-id">Provider ID *</Label>
              <Input id="prov-id" placeholder="e.g., coursera" value={newProv.id} onChange={(e) => setNewProv({ ...newProv, id: e.target.value.replace(/\s+/g, '_').toLowerCase() })} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="prov-name">Provider Name *</Label>
              <Input id="prov-name" placeholder="e.g., Coursera" value={newProv.name} onChange={(e) => setNewProv({ ...newProv, name: e.target.value })} />
            </div>
          </div>
          {error && <div className="p-3 bg-destructive/10 border border-destructive/20 rounded text-destructive text-sm">{error}</div>}
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddOpen(false)}>Cancel</Button>
            <Button onClick={addProvider} disabled={!newProv.id || !newProv.name}>Add Provider</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}


