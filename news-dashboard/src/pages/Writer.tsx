import { useEffect, useState } from 'react';
import axios from 'axios';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Globe2 } from 'lucide-react';

interface SiteItem { id: string; name: string; writer?: { provider?: string; enabled?: boolean; api_key?: string | null }; }

type Provider = { id: string; name: string; note?: string };

const ARTICLE_PROVIDERS: Provider[] = [
  { id: 'deepseek', name: 'DeepSeek' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
];

// Social uses a single API (same model as Article, no per-network selection)
const SOCIAL_PROVIDERS: Provider[] = [
  { id: 'deepseek', name: 'DeepSeek' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
];

// News uses a single API (same model as Article, for news article generation)
const NEWS_PROVIDERS: Provider[] = [
  { id: 'deepseek', name: 'DeepSeek' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
];

export default function Writer() {
  const [sites, setSites] = useState<SiteItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [updating, setUpdating] = useState<string | null>(null);
  const [editingSites, setEditingSites] = useState<string | null>(null);
  const [articleApiKey, setArticleApiKey] = useState<Record<string, string>>({});
  const [socialApiKey, setSocialApiKey] = useState<Record<string, string>>({});
  const [newsApiKey, setNewsApiKey] = useState<Record<string, string>>({});
  const [selectedSitesByProvider, setSelectedSitesByProvider] = useState<Record<string, string[]>>({});

  useEffect(() => { loadSites(); }, []);

  const loadSites = async () => {
    try {
      setLoading(true);
      setError(''); // Clear previous errors
      const resp = await axios.get('/api/sites');
      console.log('📡 API Response:', resp.data);
      if (resp.data?.success) {
        const sitesList = resp.data.sites || [];
        console.log(`📋 Loaded ${sitesList.length} sites:`, sitesList.map((s: any) => ({ id: s.id, name: s.name })));
        setSites(sitesList);
        // Prefill api keys by reading first site per provider (best effort)
        const ak: Record<string, string> = {};
        const sk: Record<string, string> = {};
        const nk: Record<string, string> = {};
        const selectedByProvider: Record<string, string[]> = {};
        
        (resp.data.sites || []).forEach((s: any) => {
          // Article providers
          if (s?.writer?.provider && s?.writer?.api_key && !ak[s.writer.provider]) {
            ak[s.writer.provider] = s.writer.api_key;
          }
          // Social providers
          if (s?.writer?.provider && s?.writer?.api_key && !sk[s.writer.provider]) {
            sk[s.writer.provider] = s.writer.api_key;
          }
          // News providers
          if (s?.writer?.provider && s?.writer?.api_key && !nk[s.writer.provider]) {
            nk[s.writer.provider] = s.writer.api_key;
          }
          // Pre-populate selectedSitesByProvider with already assigned sites
          if (s?.writer?.provider && s?.writer?.enabled !== false) {
            const provId = s.writer.provider;
            if (!selectedByProvider[provId]) {
              selectedByProvider[provId] = [];
            }
            selectedByProvider[provId].push(s.id);
          }
        });
        setArticleApiKey(ak);
        setSocialApiKey(sk);
        setNewsApiKey(nk);
        setSelectedSitesByProvider(prev => ({ ...prev, ...selectedByProvider }));
      } else {
        console.warn('⚠️ API returned success=false or no sites:', resp.data);
        setError(resp.data?.error || 'No sites available');
        setSites([]); // Ensure sites is empty if API failed
      }
    } catch (err: any) {
      console.error('❌ Error loading sites:', err);
      setError(err.response?.data?.error || err.message || 'Failed to load sites');
      setSites([]); // Set empty array on error
    } finally { setLoading(false); }
  };

  const assignedSitesForArticle = (providerId: string) => {
    return sites.filter(s => (s as any).writer?.provider === providerId && (s as any).writer?.enabled !== false)
      .map(s => ({ id: s.id, name: s.name }));
  };

  // Social currently reuses the writer provider to generate posts; reflect that here
  const assignedSitesForSocial = (providerId: string) => {
    return sites
      .filter(s => (s as any).writer?.provider === providerId && (s as any).writer?.enabled !== false)
      .map(s => ({ id: s.id, name: s.name }));
  };

  // News uses the writer provider to generate news articles; similar to Article
  const assignedSitesForNews = (providerId: string) => {
    return sites
      .filter(s => (s as any).writer?.provider === providerId && (s as any).writer?.enabled !== false)
      .map(s => ({ id: s.id, name: s.name }));
  };

  const toggleArticleProviderForSite = async (providerId: string, siteId: string, enable?: boolean) => {
    try {
      setUpdating(`${providerId}:${siteId}`);
      const payload: any = { 
        provider: providerId, 
        enabled: enable ?? true 
      };
      // Include API key if provided
      if (articleApiKey[providerId]) {
        payload.api_key = articleApiKey[providerId];
      }
      const response = await axios.put(`/api/sites/${siteId}/writer`, payload);
      if (response.data?.success) {
        setError(''); // Clear error on success
      } else {
        throw new Error(response.data?.error || 'Update failed');
      }
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || err.message || 'Failed to update writer provider';
      setError(errorMsg);
      throw err; // Re-throw so caller can handle
    } finally { 
      setUpdating(null); 
    }
  };

  const toggleSocialProviderForSite = async (providerId: string, siteId: string, enabled: boolean) => {
    try {
      setUpdating(`${providerId}:${siteId}`);
      const payload: any = { 
        provider: providerId, 
        enabled 
      };
      // Include API key if provided
      if (socialApiKey[providerId]) {
        payload.api_key = socialApiKey[providerId];
      }
      // Reuse writer endpoint to store social API provider and key for now
      await axios.put(`/api/sites/${siteId}/writer`, payload);
      await loadSites();
      setError(''); // Clear error on success
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update social status');
    } finally { setUpdating(null); }
  };

  const toggleNewsProviderForSite = async (providerId: string, siteId: string, enable?: boolean) => {
    try {
      setUpdating(`${providerId}:${siteId}`);
      const payload: any = { 
        provider: providerId, 
        enabled: enable ?? true 
      };
      // Include API key if provided
      if (newsApiKey[providerId]) {
        payload.api_key = newsApiKey[providerId];
      }
      const response = await axios.put(`/api/sites/${siteId}/writer`, payload);
      if (response.data?.success) {
        setError(''); // Clear error on success
      } else {
        throw new Error(response.data?.error || 'Update failed');
      }
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || err.message || 'Failed to update news provider';
      setError(errorMsg);
      throw err; // Re-throw so caller can handle
    } finally { 
      setUpdating(null); 
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
    <div className="p-8 space-y-8">
      {error && (
        <div className="p-3 bg-destructive/10 border border-destructive/20 rounded text-destructive">{error}</div>
      )}

      {/* Article subsection - same layout pattern as Sources */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-bold">Article</h2>
          <p className="text-muted-foreground">Select which Writer API to use for article generation per article</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {ARTICLE_PROVIDERS.map((prov) => {
            const assigned = assignedSitesForArticle(prov.id);
            const active = assigned.length > 0;
            return (
              <Card key={prov.id} className="hover-lift animate-fade-in-up">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">{prov.name}{' '}
                      <span className={`text-xs px-2 py-0.5 rounded ${active ? 'bg-green-500/10 text-green-700 dark:text-green-400' : 'border border-border text-muted-foreground'}`}>
                        {active ? 'Active' : 'Inactive'}
                      </span>
                    </CardTitle>
                  </div>
                  <CardDescription>ID: <code className="text-xs bg-muted px-1 rounded">{prov.id}</code></CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* API Key row like Sources */}
                  <div className="text-sm text-muted-foreground">
                    API Key:{' '}
                    {articleApiKey[prov.id] ? (
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
                      value={articleApiKey[prov.id] || ''}
                      onChange={(e) => setArticleApiKey({ ...articleApiKey, [prov.id]: e.target.value })}
                    />
                    <div className="flex gap-2">
                      <Button
                        variant={active ? 'destructive' : 'default'}
                        onClick={async () => {
                          // Use selected sites or fallback to already assigned sites
                          let targets = selectedSitesByProvider[prov.id] || [];
                          // If no sites selected but provider is active, use assigned sites to disable
                          if (targets.length === 0 && active) {
                            targets = assignedSitesForArticle(prov.id).map(s => s.id);
                          }
                          // If no sites selected and provider is inactive, nothing to do
                          if (targets.length === 0 && !active) {
                            setError('Please select at least one site first, then click Enable');
                            return;
                          }
                          // Save API key first if provided
                          if (articleApiKey[prov.id]) {
                            // API key will be saved with each site update
                          }
                          // Enable/disable for selected sites
                          let successCount = 0;
                          let errors: string[] = [];
                          for (const siteId of targets) {
                            try {
                              await toggleArticleProviderForSite(prov.id, siteId, !active);
                              successCount++;
                            } catch (err: any) {
                              const errorMsg = err.response?.data?.error || err.message || 'Unknown error';
                              errors.push(`${siteId}: ${errorMsg}`);
                              console.error(`Failed to update site ${siteId}:`, err);
                            }
                          }
                          
                          if (successCount > 0) {
                            // Reload sites to get updated state
                            await loadSites();
                            // Clear selection after successful save
                            setSelectedSitesByProvider(prev => {
                              const next = { ...prev };
                              delete next[prov.id];
                              return next;
                            });
                            // Close edit mode
                            setEditingSites(null);
                            setError(''); // Clear errors on success
                          } else if (errors.length > 0) {
                            setError(`Failed to update: ${errors.join('; ')}`);
                          }
                        }}
                        disabled={updating !== null || (editingSites === prov.id && (selectedSitesByProvider[prov.id] || []).length === 0 && !active)}
                      >
                        {updating ? 'Saving...' : active ? 'Disable' : 'Enable'}
                      </Button>
                      {editingSites === prov.id && (selectedSitesByProvider[prov.id] || []).length > 0 && (
                        <Badge variant="outline" className="self-center text-xs">
                          {(selectedSitesByProvider[prov.id] || []).length} selected
                        </Badge>
                      )}
                    </div>
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
                        {loading ? (
                          <p className="text-xs text-muted-foreground italic text-center py-2">Loading sites...</p>
                        ) : sites.length > 0 ? (
                          <>
                            {sites.map((s) => {
                              const selected = (selectedSitesByProvider[prov.id] || []).includes(s.id);
                              const isAssigned = assignedSitesForArticle(prov.id).some(as => as.id === s.id);
                              return (
                                <label key={s.id} className="flex items-center gap-3 p-2 rounded-md hover:bg-background cursor-pointer transition-colors border border-transparent hover:border-primary/20">
                                  <input
                                    type="checkbox"
                                    checked={selected}
                                    onChange={() => {
                                      const curr = selectedSitesByProvider[prov.id] || [];
                                      const next = selected ? curr.filter(id => id !== s.id) : [...curr, s.id];
                                      setSelectedSitesByProvider({ ...selectedSitesByProvider, [prov.id]: next });
                                      setError(''); // Clear error when selecting
                                    }}
                                    className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                                    disabled={updating !== null}
                                  />
                                  <span className="text-sm font-medium">{s.name}</span>
                                  {isAssigned && (
                                    <Badge variant="secondary" className="text-xs ml-auto">
                                      Currently assigned
                                    </Badge>
                                  )}
                                </label>
                              );
                            })}
                            <div className="pt-2 border-t border-border mt-2">
                              <p className="text-xs text-muted-foreground">
                                Selected: {(selectedSitesByProvider[prov.id] || []).length} site(s)
                              </p>
                            </div>
                          </>
                        ) : (
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

      {/* Social subsection - same layout pattern as Sources */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-bold">Social</h2>
          <p className="text-muted-foreground">Select which Writer API to use for social generation per article</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {SOCIAL_PROVIDERS.map((prov) => {
            const assigned = assignedSitesForSocial(prov.id);
            const active = assigned.length > 0;
            return (
              <Card key={prov.id} className="hover-lift animate-fade-in-up">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">{prov.name}{' '}
                      <span className={`text-xs px-2 py-0.5 rounded ${active ? 'bg-green-500/10 text-green-700 dark:text-green-400' : 'border border-border text-muted-foreground'}`}>
                        {active ? 'Active' : 'Inactive'}
                      </span>
                    </CardTitle>
                  </div>
                  <CardDescription>ID: <code className="text-xs bg-muted px-1 rounded">{prov.id}</code></CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* API Key row */}
                  <div className="text-sm text-muted-foreground">
                    API Key:{' '}
                    {socialApiKey[prov.id] ? (
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
                      value={socialApiKey[prov.id] || ''}
                      onChange={(e) => setSocialApiKey({ ...socialApiKey, [prov.id]: e.target.value })}
                    />
                    <Button
                      variant={active ? 'destructive' : 'default'}
                      onClick={async () => {
                        // Use selected sites or fallback to already assigned sites
                        let targets = selectedSitesByProvider[`social:${prov.id}`] || [];
                        // If no sites selected but provider is active, use assigned sites to disable
                        if (targets.length === 0 && active) {
                          targets = assignedSitesForSocial(prov.id).map(s => s.id);
                        }
                        // If no sites selected and provider is inactive, nothing to do
                        if (targets.length === 0 && !active) {
                          setError('Please select at least one site to enable this provider');
                          return;
                        }
                        // Enable/disable for selected sites
                        let successCount = 0;
                        for (const siteId of targets) {
                          try {
                            await toggleSocialProviderForSite(prov.id, siteId, !active);
                            successCount++;
                          } catch (err) {
                            console.error(`Failed to update site ${siteId}:`, err);
                          }
                        }
                        
                        if (successCount > 0) {
                          // Reload sites to get updated state
                          await loadSites();
                          // Clear selection after successful save
                          setSelectedSitesByProvider(prev => {
                            const next = { ...prev };
                            delete next[`social:${prov.id}`];
                            return next;
                          });
                          // Close edit mode
                          setEditingSites(null);
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
                        onClick={() => setEditingSites(editingSites === `social:${prov.id}` ? null : `social:${prov.id}`)}
                        className="h-7 px-3 text-xs"
                        disabled={updating !== null}
                      >
                        {editingSites === `social:${prov.id}` ? 'Cancel' : 'Select Sites'}
                      </Button>
                    </div>
                    {editingSites === `social:${prov.id}` ? (
                      <div className="space-y-2 bg-muted/30 p-3 rounded-lg border border-border">
                        {sites.map((s) => {
                          const selected = (selectedSitesByProvider[`social:${prov.id}`] || []).includes(s.id);
                          return (
                            <label key={s.id} className="flex items-center gap-3 p-2 rounded-md hover:bg-background cursor-pointer transition-colors border border-transparent hover:border-primary/20">
                              <input
                                type="checkbox"
                                checked={selected}
                                onChange={() => {
                                  const key = `social:${prov.id}`;
                                  const curr = selectedSitesByProvider[key] || [];
                                  const next = selected ? curr.filter(id => id !== s.id) : [...curr, s.id];
                                  setSelectedSitesByProvider({ ...selectedSitesByProvider, [key]: next });
                                }}
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

      {/* News subsection - same layout pattern as Article and Social */}
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-bold">News</h2>
          <p className="text-muted-foreground">Select which Writer API to use for news article generation per site</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {NEWS_PROVIDERS.map((prov) => {
            const assigned = assignedSitesForNews(prov.id);
            const active = assigned.length > 0;
            return (
              <Card key={prov.id} className="hover-lift animate-fade-in-up">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">{prov.name}{' '}
                      <span className={`text-xs px-2 py-0.5 rounded ${active ? 'bg-green-500/10 text-green-700 dark:text-green-400' : 'border border-border text-muted-foreground'}`}>
                        {active ? 'Active' : 'Inactive'}
                      </span>
                    </CardTitle>
                  </div>
                  <CardDescription>ID: <code className="text-xs bg-muted px-1 rounded">{prov.id}</code></CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* API Key row like Sources */}
                  <div className="text-sm text-muted-foreground">
                    API Key:{' '}
                    {newsApiKey[prov.id] ? (
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
                      value={newsApiKey[prov.id] || ''}
                      onChange={(e) => setNewsApiKey({ ...newsApiKey, [prov.id]: e.target.value })}
                    />
                    <div className="flex gap-2">
                      <Button
                        variant={active ? 'destructive' : 'default'}
                        onClick={async () => {
                          // Use selected sites or fallback to already assigned sites
                          let targets = selectedSitesByProvider[`news:${prov.id}`] || [];
                          // If no sites selected but provider is active, use assigned sites to disable
                          if (targets.length === 0 && active) {
                            targets = assignedSitesForNews(prov.id).map(s => s.id);
                          }
                          // If no sites selected and provider is inactive, nothing to do
                          if (targets.length === 0 && !active) {
                            setError('Please select at least one site first, then click Enable');
                            return;
                          }
                          // Save API key first if provided
                          if (newsApiKey[prov.id]) {
                            // API key will be saved with each site update
                          }
                          // Enable/disable for selected sites
                          let successCount = 0;
                          let errors: string[] = [];
                          for (const siteId of targets) {
                            try {
                              await toggleNewsProviderForSite(prov.id, siteId, !active);
                              successCount++;
                            } catch (err: any) {
                              const errorMsg = err.response?.data?.error || err.message || 'Unknown error';
                              errors.push(`${siteId}: ${errorMsg}`);
                              console.error(`Failed to update site ${siteId}:`, err);
                            }
                          }
                          
                          if (successCount > 0) {
                            // Reload sites to get updated state
                            await loadSites();
                            // Clear selection after successful save
                            setSelectedSitesByProvider(prev => {
                              const next = { ...prev };
                              delete next[`news:${prov.id}`];
                              return next;
                            });
                            // Close edit mode
                            setEditingSites(null);
                            setError(''); // Clear errors on success
                          } else if (errors.length > 0) {
                            setError(`Failed to update: ${errors.join('; ')}`);
                          }
                        }}
                        disabled={updating !== null || (editingSites === `news:${prov.id}` && (selectedSitesByProvider[`news:${prov.id}`] || []).length === 0 && !active)}
                      >
                        {updating ? 'Saving...' : active ? 'Disable' : 'Enable'}
                      </Button>
                      {editingSites === `news:${prov.id}` && (selectedSitesByProvider[`news:${prov.id}`] || []).length > 0 && (
                        <Badge variant="outline" className="self-center text-xs">
                          {(selectedSitesByProvider[`news:${prov.id}`] || []).length} selected
                        </Badge>
                      )}
                    </div>
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
                        onClick={() => setEditingSites(editingSites === `news:${prov.id}` ? null : `news:${prov.id}`)}
                        className="h-7 px-3 text-xs"
                        disabled={updating !== null}
                      >
                        {editingSites === `news:${prov.id}` ? 'Cancel' : 'Select Sites'}
                      </Button>
                    </div>
                    {editingSites === `news:${prov.id}` ? (
                      <div className="space-y-2 bg-muted/30 p-3 rounded-lg border border-border">
                        {loading ? (
                          <p className="text-xs text-muted-foreground italic text-center py-2">Loading sites...</p>
                        ) : sites.length > 0 ? (
                          <>
                            {sites.map((s) => {
                              const selected = (selectedSitesByProvider[`news:${prov.id}`] || []).includes(s.id);
                              const isAssigned = assignedSitesForNews(prov.id).some(as => as.id === s.id);
                              return (
                                <label key={s.id} className="flex items-center gap-3 p-2 rounded-md hover:bg-background cursor-pointer transition-colors border border-transparent hover:border-primary/20">
                                  <input
                                    type="checkbox"
                                    checked={selected}
                                    onChange={() => {
                                      const key = `news:${prov.id}`;
                                      const curr = selectedSitesByProvider[key] || [];
                                      const next = selected ? curr.filter(id => id !== s.id) : [...curr, s.id];
                                      setSelectedSitesByProvider({ ...selectedSitesByProvider, [key]: next });
                                      setError(''); // Clear error when selecting
                                    }}
                                    className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                                    disabled={updating !== null}
                                  />
                                  <span className="text-sm font-medium">{s.name}</span>
                                  {isAssigned && (
                                    <Badge variant="secondary" className="text-xs ml-auto">
                                      Currently assigned
                                    </Badge>
                                  )}
                                </label>
                              );
                            })}
                            <div className="pt-2 border-t border-border mt-2">
                              <p className="text-xs text-muted-foreground">
                                Selected: {(selectedSitesByProvider[`news:${prov.id}`] || []).length} site(s)
                              </p>
                            </div>
                          </>
                        ) : (
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
    </div>
  );
}


