import { useState, useEffect } from 'react';
import axios from 'axios';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select-advanced";
import { Upload, Save, Eye, EyeOff, Trash2, Edit, ExternalLink } from "lucide-react";

interface PromoArticle {
  id: string;
  site: 'airesearch' | 'scienceai';
  title: string;
  subtitle: string;
  content: string;
  image_url?: string;
  external_link?: string;
  featured: boolean;
  hidden: boolean;
  created_at: string;
  updated_at: string;
}

export default function Promo() {
  const [articles, setArticles] = useState<PromoArticle[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  
  // Form state
  const [formData, setFormData] = useState({
    site: 'airesearch' as 'airesearch' | 'scienceai',
    title: '',
    subtitle: '',
    content: '',
    external_link: '',
  });
  const [imageFile, setImageFile] = useState<File | null>(null);
  const [imagePreview, setImagePreview] = useState<string>('');
  const [saving, setSaving] = useState(false);

  const loadArticles = async () => {
    try {
      setLoading(true);
      setError('');
      const res = await axios.get('/api/promo/articles');
      if (res.data?.success) {
        setArticles(res.data.articles || []);
      } else {
        setError(res.data?.error || 'Failed to load promo articles');
      }
    } catch (e: any) {
      setError(e.response?.data?.error || e.message || 'Failed to load promo articles');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadArticles();
  }, []);

  const resetForm = () => {
    setFormData({
      site: 'airesearch',
      title: '',
      subtitle: '',
      content: '',
      external_link: '',
    });
    setImageFile(null);
    setImagePreview('');
    setEditingId(null);
  };

  const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      // Validate file type
      if (!file.type.startsWith('image/')) {
        setError('Please select a valid image file');
        return;
      }
      
      // Validate file size (max 5MB)
      if (file.size > 5 * 1024 * 1024) {
        setError('Image size must be less than 5MB');
        return;
      }

      setImageFile(file);
      
      // Create preview
      const reader = new FileReader();
      reader.onload = (e) => {
        setImagePreview(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const handleSave = async () => {
    if (!formData.title.trim() || !formData.subtitle.trim() || !formData.content.trim()) {
      setError('Title, subtitle, and content are required');
      return;
    }

    setSaving(true);
    setError('');

    try {
      const formDataToSend = new FormData();
      formDataToSend.append('site', formData.site);
      formDataToSend.append('title', formData.title.trim());
      formDataToSend.append('subtitle', formData.subtitle.trim());
      // Preserve paragraph structure - only trim start/end, keep internal formatting
      formDataToSend.append('content', formData.content.replace(/^\s+|\s+$/g, ''));
      formDataToSend.append('external_link', formData.external_link.trim());
      formDataToSend.append('featured', 'true'); // Always featured for promo articles
      
      if (imageFile) {
        formDataToSend.append('image', imageFile);
      }

      let response;
      if (editingId) {
        response = await axios.put(`/api/promo/articles/${editingId}`, formDataToSend, {
          headers: { 'Content-Type': 'multipart/form-data' }
        });
      } else {
        response = await axios.post('/api/promo/articles', formDataToSend, {
          headers: { 'Content-Type': 'multipart/form-data' }
        });
      }

      if (response.data?.success) {
        await loadArticles(); // Reload articles
        resetForm();
      } else {
        setError(response.data?.error || 'Failed to save article');
      }
    } catch (e: any) {
      setError(e.response?.data?.error || e.message || 'Failed to save article');
    } finally {
      setSaving(false);
    }
  };

  const handleToggleVisibility = async (id: string, currentHidden: boolean) => {
    try {
      const response = await axios.put(`/api/promo/articles/${id}/visibility`, {
        hidden: !currentHidden
      });
      
      if (response.data?.success) {
        setArticles(prev => prev.map(article => 
          article.id === id ? { ...article, hidden: !currentHidden } : article
        ));
      } else {
        setError(response.data?.error || 'Failed to update visibility');
      }
    } catch (e: any) {
      setError(e.response?.data?.error || e.message || 'Failed to update visibility');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this promo article?')) {
      return;
    }

    try {
      const response = await axios.delete(`/api/promo/articles/${id}`);
      
      if (response.data?.success) {
        setArticles(prev => prev.filter(article => article.id !== id));
      } else {
        setError(response.data?.error || 'Failed to delete article');
      }
    } catch (e: any) {
      setError(e.response?.data?.error || e.message || 'Failed to delete article');
    }
  };

  const handleEdit = (article: PromoArticle) => {
    setFormData({
      site: article.site,
      title: article.title,
      subtitle: article.subtitle,
      content: article.content,
      external_link: article.external_link || '',
    });
    setImagePreview(article.image_url || '');
    setEditingId(article.id);
    setImageFile(null);
  };

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Promotional Articles</h1>
        <p className="text-muted-foreground mt-2">Create and manage promotional articles for featured display</p>
      </div>

      {error && (
        <div className="bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 rounded-lg">
          {error}
        </div>
      )}

      {/* Create/Edit Form */}
      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>{editingId ? 'Edit' : 'Create'} Promotional Article</CardTitle>
          <CardDescription>
            {editingId ? 'Update the promotional article' : 'Create a new promotional article that will be featured prominently'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="site">Target Site</Label>
                <Select value={formData.site} onValueChange={(value: string) => 
                  setFormData(prev => ({ ...prev, site: value as 'airesearch' | 'scienceai' }))
                }>
                  <SelectTrigger>
                    <SelectValue placeholder="Select target site" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="airesearch">AIResearch (Featured Articles)</SelectItem>
                    <SelectItem value="scienceai">ScienceAI (Carousel)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="title">Title</Label>
                <Input
                  id="title"
                  value={formData.title}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData(prev => ({ ...prev, title: e.target.value }))}
                  placeholder="Enter article title..."
                  maxLength={100}
                />
                <p className="text-xs text-muted-foreground mt-1">{formData.title.length}/100 characters</p>
              </div>

              <div>
                <Label htmlFor="subtitle">Subtitle</Label>
                <Input
                  id="subtitle"
                  value={formData.subtitle}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData(prev => ({ ...prev, subtitle: e.target.value }))}
                  placeholder="Enter article subtitle..."
                  maxLength={200}
                />
                <p className="text-xs text-muted-foreground mt-1">{formData.subtitle.length}/200 characters</p>
              </div>

              <div>
                <Label htmlFor="external_link">External Link (Optional)</Label>
                <Input
                  id="external_link"
                  type="url"
                  value={formData.external_link}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData(prev => ({ ...prev, external_link: e.target.value }))}
                  placeholder="https://example.com/read-more"
                />
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <Label htmlFor="image">Article Image</Label>
                <div className="border-2 border-dashed border-border rounded-lg p-6 text-center">
                  {imagePreview ? (
                    <div className="space-y-4">
                      <img 
                        src={imagePreview} 
                        alt="Preview" 
                        className="max-w-full h-48 object-cover rounded-lg mx-auto"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          setImagePreview('');
                          setImageFile(null);
                        }}
                      >
                        Remove Image
                      </Button>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      <Upload className="mx-auto h-12 w-12 text-muted-foreground" />
                      <div>
                        <Label htmlFor="image-upload" className="cursor-pointer">
                          <span className="text-primary hover:text-primary/80">Click to upload</span>
                          <span className="text-muted-foreground"> or drag and drop</span>
                        </Label>
                        <p className="text-xs text-muted-foreground mt-1">
                          PNG, JPG, WebP up to 5MB
                        </p>
                      </div>
                    </div>
                  )}
                  <Input
                    id="image-upload"
                    type="file"
                    accept="image/*"
                    onChange={handleImageChange}
                    className="hidden"
                  />
                </div>
              </div>
            </div>
          </div>

          <div>
            <Label htmlFor="content">Article Content</Label>
            <Textarea
              id="content"
              value={formData.content}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setFormData(prev => ({ ...prev, content: e.target.value }))}
              placeholder="Enter the full article content... (Separate paragraphs with double line breaks)"
              rows={8}
              maxLength={5000}
              style={{ whiteSpace: 'pre-wrap' }}
            />
            <p className="text-xs text-muted-foreground mt-1">
              {formData.content.length}/5000 characters
              {formData.content.includes('\n\n') && (
                <span className="text-green-600 ml-2">✓ Paragraphs detected</span>
              )}
            </p>
          </div>

          <div className="flex gap-3">
            <Button 
              onClick={handleSave} 
              disabled={saving || !formData.title.trim() || !formData.subtitle.trim() || !formData.content.trim()}
              className="flex items-center gap-2"
            >
              <Save size={16} />
              {saving ? 'Saving...' : editingId ? 'Update Article' : 'Save & Publish'}
            </Button>
            
            {editingId && (
              <Button 
                variant="outline" 
                onClick={resetForm}
                className="flex items-center gap-2"
              >
                Cancel Edit
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Articles List */}
      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>Existing Promotional Articles</CardTitle>
          <CardDescription>Manage your promotional articles</CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="p-6 text-center text-muted-foreground">Loading articles...</div>
          ) : articles.length === 0 ? (
            <div className="p-6 text-center text-muted-foreground">No promotional articles found</div>
          ) : (
            <div className="space-y-4">
              {articles.map(article => (
                <div key={article.id} className="border rounded-lg p-4 space-y-3">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="font-semibold text-lg">{article.title}</h3>
                        <Badge variant={article.site === 'airesearch' ? 'default' : 'secondary'}>
                          {article.site === 'airesearch' ? 'AIResearch' : 'ScienceAI'}
                        </Badge>
                        {article.featured && <Badge variant="outline">Featured</Badge>}
                        {article.hidden && <Badge variant="destructive">Hidden</Badge>}
                      </div>
                      <p className="text-muted-foreground mb-2">{article.subtitle}</p>
                      <p className="text-sm text-muted-foreground line-clamp-2">{article.content}</p>
                      {article.external_link && (
                        <a 
                          href={article.external_link} 
                          target="_blank" 
                          rel="noopener noreferrer"
                          className="inline-flex items-center gap-1 text-sm text-primary hover:text-primary/80 mt-2"
                        >
                          <ExternalLink size={14} />
                          Read More
                        </a>
                      )}
                    </div>
                    
                    {article.image_url && (
                      <img 
                        src={article.image_url} 
                        alt={article.title}
                        className="w-24 h-24 object-cover rounded-lg ml-4"
                      />
                    )}
                  </div>
                  
                  <div className="flex items-center gap-2 pt-2 border-t">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleEdit(article)}
                      className="flex items-center gap-1"
                    >
                      <Edit size={14} />
                      Edit
                    </Button>
                    
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => handleToggleVisibility(article.id, article.hidden)}
                      className="flex items-center gap-1"
                    >
                      {article.hidden ? <Eye size={14} /> : <EyeOff size={14} />}
                      {article.hidden ? 'Show' : 'Hide'}
                    </Button>
                    
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={() => handleDelete(article.id)}
                      className="flex items-center gap-1"
                    >
                      <Trash2 size={14} />
                      Delete
                    </Button>
                    
                    <div className="ml-auto text-xs text-muted-foreground">
                      Created: {new Date(article.created_at).toLocaleDateString()}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
