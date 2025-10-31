import { useState, useEffect } from "react";
import { Plus, Edit, Trash2, Power, Settings, Save, X } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import axios from "axios";

interface Page {
  id: string;
  name: string;
  sources: string[];
  frequency_minutes: number;
  writing_style: string;
  active: boolean;
  domain?: string | null;
  is_online?: boolean;
}

export default function PagesConfig() {
  const [pages, setPages] = useState<Page[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [editingFrequency, setEditingFrequency] = useState<string | null>(null);
  const [tempFrequency, setTempFrequency] = useState<number>(60);
  const [updating, setUpdating] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newPage, setNewPage] = useState({
    id: "",
    name: "",
    domain: "",
    frequency_minutes: 60,
    writing_style: "scientific",
    enabled: true,
  });

  useEffect(() => {
    loadPages();
  }, []);

  const loadPages = async () => {
    try {
      setLoading(true);
      const response = await axios.get('/api/pages');
      
      if (response.data.success) {
        setPages(response.data.pages || []);
      } else {
        setError(response.data.error || 'Failed to load pages');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to load pages');
    } finally {
      setLoading(false);
    }
  };

  const startEditingFrequency = (pageId: string, currentFrequency: number) => {
    setEditingFrequency(pageId);
    setTempFrequency(currentFrequency);
  };

  const cancelEditingFrequency = () => {
    setEditingFrequency(null);
    setTempFrequency(60);
  };

  const saveFrequency = async (pageId: string) => {
    try {
      setUpdating(pageId);
      const response = await axios.put(`/api/pages/${pageId}`, {
        frequency_minutes: tempFrequency,
      });

      if (response.data.success) {
        setPages(prev =>
          prev.map(p =>
            p.id === pageId ? { ...p, frequency_minutes: tempFrequency } : p
          )
        );
        setEditingFrequency(null);
      } else {
        setError(response.data.error || 'Failed to update frequency');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update frequency');
    } finally {
      setUpdating(null);
    }
  };

  const togglePageStatus = async (pageId: string, currentStatus: boolean) => {
    try {
      setUpdating(pageId);
      const response = await axios.put(`/api/pages/${pageId}`, {
        enabled: !currentStatus,
      });

      if (response.data.success) {
        setPages(prev =>
          prev.map(p =>
            p.id === pageId ? { ...p, active: !currentStatus } : p
          )
        );
      } else {
        setError(response.data.error || 'Failed to update page status');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update page status');
    } finally {
      setUpdating(null);
    }
  };

  const handleOpenDialog = () => {
    setNewPage({
      id: "",
      name: "",
      domain: "",
      frequency_minutes: 60,
      writing_style: "scientific",
      enabled: true,
    });
    setError(""); // Clear any previous errors
    setDialogOpen(true);
  };

  const handleCloseDialog = () => {
    setDialogOpen(false);
    setError("");
  };

  const handleCreatePage = async () => {
    // Validate form
    if (!newPage.id.trim()) {
      setError("Page ID is required");
      return;
    }
    if (!newPage.name.trim()) {
      setError("Page name is required");
      return;
    }
    
    // Validate ID format (lowercase, no spaces, alphanumeric and underscore only)
    const idPattern = /^[a-z0-9_]+$/;
    if (!idPattern.test(newPage.id)) {
      setError("Page ID must be lowercase with no spaces (letters, numbers, and underscores only)");
      return;
    }

    try {
      setCreating(true);
      setError("");

      const payload = {
        id: newPage.id.trim(),
        name: newPage.name.trim(),
        domain: newPage.domain.trim() || null,
        frequency_minutes: newPage.frequency_minutes,
        writing_style: newPage.writing_style,
        enabled: newPage.enabled,
      };

      console.log("Sending request to create page:", payload);

      const response = await axios.post("/api/pages", payload);

      console.log("Response received:", response);
      console.log("Response data:", response.data);
      console.log("Response status:", response.status);

      if (response.data && response.data.success) {
        console.log("Page created successfully");
        setDialogOpen(false);
        setError(""); // Clear any previous errors
        await loadPages(); // Reload pages list
      } else if (response.status === 200 && typeof response.data?.message === 'string' && response.data.message.toLowerCase().includes('create page endpoint')) {
        // Fallback: some older backends return a placeholder for POST /api/pages
        // Attempt an upsert using PUT /api/pages/:id so the user can proceed
        console.warn('POST /api/pages returned placeholder. Falling back to PUT /api/pages/:id');
        const putResp = await axios.put(`/api/pages/${payload.id}`, {
          frequency_minutes: payload.frequency_minutes,
          writing_style: payload.writing_style,
          enabled: payload.enabled,
        });
        if (putResp.data?.success) {
          console.log('Fallback PUT succeeded');
          setDialogOpen(false);
          setError("");
          await loadPages();
        } else {
          const err2 = putResp.data?.error || putResp.data?.message || 'Failed to upsert page';
          console.error('Fallback PUT error:', err2);
          setError(err2);
        }
      } else {
        const errorMsg = response.data?.error || response.data?.message || "Failed to create page";
        console.error("Backend returned error:", errorMsg);
        setError(errorMsg);
      }
    } catch (err: any) {
      console.error("Exception caught while creating page:", err);
      console.error("Error type:", typeof err);
      console.error("Error response:", err.response);
      console.error("Error response data:", err.response?.data);
      console.error("Error response status:", err.response?.status);
      console.error("Error message:", err.message);
      
      const errorMessage = err.response?.data?.error 
        || err.response?.data?.message
        || err.message 
        || "Failed to create page";
      
      console.error("Setting error message:", errorMessage);
      setError(errorMessage);
    } finally {
      setCreating(false);
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
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Pages Configuration</h1>
          <p className="text-muted-foreground mt-2">Manage your content portals</p>
        </div>
        <Button variant="default" className="gap-2 hover-lift" onClick={handleOpenDialog}>
          <Plus size={20} />
          Add Page
        </Button>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive">
          {error}
        </div>
      )}

      {/* Pages List */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {pages.map((page) => (
          <Card key={page.id} className="hover-lift animate-fade-in-up">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 flex-wrap">
                  {page.name}
                  <Badge variant={page.active ? "default" : "outline"}>
                    {page.active ? "Active" : "Inactive"}
                  </Badge>
                  {(!page.is_online || !page.domain) && (
                    <Badge variant="outline" className="bg-orange-500/10 text-orange-700 dark:text-orange-400 border-orange-500/20">
                      Offline
                    </Badge>
                  )}
                </CardTitle>
                <div className="flex gap-2">
                  <Button variant="ghost" size="icon" title="Edit">
                    <Edit size={16} />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    title={page.active ? "Deactivate" : "Activate"}
                    onClick={() => togglePageStatus(page.id, page.active)}
                    disabled={updating === page.id}
                  >
                    <Power size={16} className={page.active ? "text-green-600" : ""} />
                  </Button>
                  <Button variant="ghost" size="icon" title="Delete">
                    <Trash2 size={16} className="text-destructive" />
                  </Button>
                </div>
              </div>
              <CardDescription>Content portal configuration</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Domain Status */}
              {page.domain && (
                <div className="flex items-center gap-2">
                  <p className="text-sm font-medium text-muted-foreground">Domain:</p>
                  <code className="text-xs bg-muted px-2 py-1 rounded">{page.domain}</code>
                  {!page.is_online && (
                    <Badge variant="outline" className="bg-orange-500/10 text-orange-700 dark:text-orange-400 border-orange-500/20 text-xs">
                      Offline
                    </Badge>
                  )}
                </div>
              )}
              {!page.domain && (
                <div className="flex items-center gap-2">
                  <p className="text-sm font-medium text-muted-foreground">Domain:</p>
                  <Badge variant="outline" className="bg-orange-500/10 text-orange-700 dark:text-orange-400 border-orange-500/20 text-xs">
                    Not configured - Offline
                  </Badge>
                </div>
              )}

              {/* Sources */}
              <div>
                <p className="text-sm font-medium text-muted-foreground mb-2">Sources</p>
                <div className="flex gap-2 flex-wrap">
                  {page.sources.length > 0 ? (
                    page.sources.map((source, idx) => (
                      <Badge key={idx} variant="secondary">
                        {source}
                      </Badge>
                    ))
                  ) : (
                    <span className="text-sm text-muted-foreground italic">No active sources</span>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Synced with enabled collectors from Sources page
                </p>
              </div>

              {/* Config */}
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-muted-foreground mb-1">Frequency</p>
                  {editingFrequency === page.id ? (
                    <div className="flex items-center gap-2">
                      <Input
                        type="number"
                        min="1"
                        value={tempFrequency}
                        onChange={(e) => setTempFrequency(parseInt(e.target.value) || 60)}
                        className="w-20 h-8 text-sm"
                        disabled={updating === page.id}
                      />
                      <span className="text-xs text-muted-foreground">min</span>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => saveFrequency(page.id)}
                        disabled={updating === page.id}
                        className="h-8 px-2"
                      >
                        <Save size={14} />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={cancelEditingFrequency}
                        disabled={updating === page.id}
                        className="h-8 px-2"
                      >
                        <X size={14} />
                      </Button>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{page.frequency_minutes}m</p>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => startEditingFrequency(page.id, page.frequency_minutes)}
                        className="h-6 px-2"
                      >
                        <Edit size={12} />
                      </Button>
                    </div>
                  )}
                </div>
                <div>
                  <p className="text-muted-foreground">Style</p>
                  <p className="font-medium capitalize">{page.writing_style}</p>
                </div>
              </div>

              <Button variant="outline" className="w-full gap-2">
                <Settings size={16} />
                Configure
              </Button>
            </CardContent>
          </Card>
        ))}
      </div>

      {pages.length === 0 && !loading && (
        <div className="text-center py-12 text-muted-foreground">
          <p>No pages configured yet.</p>
          <p className="text-sm mt-2">Click "Add Page" to create a new one.</p>
        </div>
      )}

      {/* Create Page Dialog */}
      <Dialog open={dialogOpen} onOpenChange={(open) => {
        setDialogOpen(open);
        if (!open) {
          setError(""); // Clear error when dialog closes
        }
      }}>
        <DialogContent onClose={handleCloseDialog} className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Create New Page</DialogTitle>
            <DialogDescription>
              Create a new content portal page. Configure sources and settings after creation.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="page-id">Page ID *</Label>
              <Input
                id="page-id"
                placeholder="e.g., technews"
                value={newPage.id}
                onChange={(e) => setNewPage({ ...newPage, id: e.target.value.toLowerCase().replace(/\s+/g, "_") })}
                disabled={creating}
              />
              <p className="text-xs text-muted-foreground">
                Unique identifier (lowercase, no spaces, letters/numbers/underscores only)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="page-name">Page Name *</Label>
              <Input
                id="page-name"
                placeholder="e.g., Tech News"
                value={newPage.name}
                onChange={(e) => setNewPage({ ...newPage, name: e.target.value })}
                disabled={creating}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="page-domain">Domain (optional)</Label>
              <Input
                id="page-domain"
                placeholder="e.g., technews.com"
                value={newPage.domain}
                onChange={(e) => setNewPage({ ...newPage, domain: e.target.value })}
                disabled={creating}
              />
              <p className="text-xs text-muted-foreground">
                Optional domain name for this page
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="page-frequency">Collection Frequency (minutes)</Label>
                <Input
                  id="page-frequency"
                  type="number"
                  min="1"
                  value={newPage.frequency_minutes}
                  onChange={(e) => setNewPage({ ...newPage, frequency_minutes: parseInt(e.target.value) || 60 })}
                  disabled={creating}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="page-style">Writing Style</Label>
                <select
                  id="page-style"
                  value={newPage.writing_style}
                  onChange={(e) => setNewPage({ ...newPage, writing_style: e.target.value })}
                  disabled={creating}
                  className="w-full h-10 px-3 rounded-md border border-input bg-background text-sm"
                >
                  <option value="scientific">Scientific</option>
                  <option value="technical">Technical</option>
                  <option value="general">General</option>
                  <option value="news">News</option>
                </select>
              </div>
            </div>

            {error && (
              <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
                {error}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={handleCloseDialog} disabled={creating}>
              Cancel
            </Button>
            <Button onClick={handleCreatePage} disabled={creating || !newPage.id.trim() || !newPage.name.trim()}>
              {creating ? "Creating..." : "Create Page"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
