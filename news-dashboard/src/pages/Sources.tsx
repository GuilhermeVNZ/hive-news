import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function Sources() {
  const sources = [
    { name: "Nature", type: "API", status: "active" },
    { name: "Science", type: "API", status: "active" },
    { name: "arXiv", type: "API", status: "active" },
    { name: "TechCrunch", type: "RSS", status: "inactive" },
  ];

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Sources</h1>
          <p className="text-muted-foreground mt-2">Manage content sources</p>
        </div>
        <Button variant="default" className="gap-2">
          <Plus size={20} />
          Add Source
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {sources.map((source, idx) => (
          <Card key={idx} className="hover-lift animate-fade-in-up">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>{source.name}</CardTitle>
                <Badge variant={source.status === "active" ? "default" : "outline"}>
                  {source.status}
                </Badge>
              </div>
              <CardDescription>Source type: {source.type}</CardDescription>
            </CardHeader>
            <CardContent>
              <Button variant="outline" className="w-full">
                Configure
              </Button>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
