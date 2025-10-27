import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { CheckCircle, XCircle, Clock } from "lucide-react";

export default function Logs() {
  const logs = [
    { id: 1, page: "AIResearch", status: "success", time: "2h ago", duration: "1.2s", articles: 12 },
    { id: 2, page: "ScienceAI", status: "error", time: "4h ago", duration: "0.8s", articles: 0 },
    { id: 3, page: "AIResearch", status: "success", time: "6h ago", duration: "1.5s", articles: 15 },
    { id: 4, page: "AIResearch", status: "success", time: "8h ago", duration: "1.1s", articles: 10 },
  ];

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Collection Logs</h1>
        <p className="text-muted-foreground mt-2">View and monitor collection activities</p>
      </div>

      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>Recent Collections</CardTitle>
          <CardDescription>Last 24 hours activity</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {logs.map((log) => (
              <div
                key={log.id}
                className="flex items-center justify-between p-4 rounded-lg border border-border hover:bg-accent/50 transition-colors"
              >
                <div className="flex items-center gap-4">
                  {log.status === "success" ? (
                    <CheckCircle size={20} className="text-green-500" />
                  ) : (
                    <XCircle size={20} className="text-red-500" />
                  )}
                  <div>
                    <p className="font-medium text-foreground">{log.page}</p>
                    <div className="flex items-center gap-3 text-sm text-muted-foreground mt-1">
                      <span>{log.time}</span>
                      <span>•</span>
                      <span>{log.duration}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <Badge variant={log.status === "success" ? "default" : "destructive"}>
                    {log.articles} articles
                  </Badge>
                  <Badge variant="outline">
                    {log.status === "success" ? "Success" : "Error"}
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card className="animate-fade-in-up">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total Collections
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">24</p>
            <p className="text-sm text-muted-foreground mt-2">Last 24 hours</p>
          </CardContent>
        </Card>

        <Card className="animate-fade-in-up">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Success Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">95%</p>
            <p className="text-sm text-muted-foreground mt-2">Last 24 hours</p>
          </CardContent>
        </Card>

        <Card className="animate-fade-in-up">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Avg Duration
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">1.2s</p>
            <p className="text-sm text-muted-foreground mt-2">Last 24 hours</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
