import { useState } from "react";
import { Plus, Edit, Trash2, Power, Settings } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

export default function PagesConfig() {
  const [pages, setPages] = useState([
    {
      id: 1,
      name: "AIResearch",
      sources: ["Nature", "Science"],
      frequency_minutes: 60,
      writing_style: "scientific",
      active: true,
    },
    {
      id: 2,
      name: "ScienceAI",
      sources: ["arXiv"],
      frequency_minutes: 120,
      writing_style: "technical",
      active: false,
    },
  ]);

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Pages Configuration</h1>
          <p className="text-muted-foreground mt-2">Manage your content portals</p>
        </div>
        <Button variant="default" className="gap-2 hover-lift">
          <Plus size={20} />
          Add Page
        </Button>
      </div>

      {/* Pages List */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {pages.map((page) => (
          <Card key={page.id} className="hover-lift animate-fade-in-up">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2">
                  {page.name}
                  <Badge variant={page.active ? "default" : "outline"}>
                    {page.active ? "Active" : "Inactive"}
                  </Badge>
                </CardTitle>
                <div className="flex gap-2">
                  <Button variant="ghost" size="icon">
                    <Edit size={16} />
                  </Button>
                  <Button variant="ghost" size="icon">
                    <Power size={16} />
                  </Button>
                  <Button variant="ghost" size="icon">
                    <Trash2 size={16} className="text-destructive" />
                  </Button>
                </div>
              </div>
              <CardDescription>Content portal configuration</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Sources */}
              <div>
                <p className="text-sm font-medium text-muted-foreground mb-2">Sources</p>
                <div className="flex gap-2 flex-wrap">
                  {page.sources.map((source, idx) => (
                    <Badge key={idx} variant="secondary">
                      {source}
                    </Badge>
                  ))}
                </div>
              </div>

              {/* Config */}
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-muted-foreground">Frequency</p>
                  <p className="font-medium">{page.frequency_minutes}m</p>
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
    </div>
  );
}
