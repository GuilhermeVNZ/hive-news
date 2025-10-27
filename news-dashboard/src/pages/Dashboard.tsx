import { useState } from "react";
import { Activity, Clock, CheckCircle, XCircle, TrendingUp, Users, FileCode } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

export default function Dashboard() {
  const [selectedPeriod, setSelectedPeriod] = useState<"today" | "week" | "month">("today");

  const stats = [
    { 
      label: "Total Pages", 
      value: "2", 
      icon: Activity, 
      color: "primary",
      change: "+12%",
      trend: "up"
    },
    { 
      label: "Active Pages", 
      value: "1", 
      icon: CheckCircle, 
      color: "green",
      change: "+3%",
      trend: "up"
    },
    { 
      label: "Last Collection", 
      value: "2h ago", 
      icon: Clock, 
      color: "yellow",
      change: "-5min",
      trend: "down"
    },
    { 
      label: "Articles Today", 
      value: "24", 
      icon: TrendingUp, 
      color: "blue",
      change: "+8",
      trend: "up"
    },
  ];

  const recentActivity = [
    { page: "AIResearch", status: "success", time: "2h ago", articles: 12 },
    { page: "ScienceAI", status: "error", time: "4h ago", articles: 0 },
    { page: "AIResearch", status: "success", time: "6h ago", articles: 15 },
  ];

  return (
    <div className="p-8 space-y-8 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Dashboard</h1>
          <p className="text-muted-foreground mt-2">Overview of your news management system</p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={selectedPeriod}
            onChange={(e) => setSelectedPeriod(e.target.value as any)}
            className="border border-input bg-background hover:bg-accent rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option value="today">Today</option>
            <option value="week">This Week</option>
            <option value="month">This Month</option>
          </select>
          <Button variant="default">
            <Activity size={16} />
            Refresh
          </Button>
        </div>
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

      {/* Recent Activity */}
      <Card className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>Recent Activity</CardTitle>
          <CardDescription>Latest collection logs from your pages</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {recentActivity.map((activity, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-4 rounded-lg border border-border hover:bg-accent/50 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className={`w-3 h-3 rounded-full ${
                    activity.status === "success" ? "bg-green-500" : "bg-red-500"
                  }`} />
                  <div>
                    <p className="font-medium text-foreground">{activity.page}</p>
                    <p className="text-sm text-muted-foreground">
                      {activity.articles} articles • {activity.time}
                    </p>
                  </div>
                </div>
                <Badge variant={activity.status === "success" ? "default" : "destructive"}>
                  {activity.status === "success" ? "Success" : "Failed"}
                </Badge>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-2">
            <Button variant="default" className="justify-start">
              <Activity size={16} className="mr-2" />
              Start Collection
            </Button>
            <Button variant="outline" className="justify-start">
              <FileCode size={16} className="mr-2" />
              Manage Pages
            </Button>
          </CardContent>
        </Card>

        <Card className="hover-lift animate-fade-in-up">
          <CardHeader>
            <CardTitle>System Status</CardTitle>
            <CardDescription>All systems operational</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Backend API</span>
                <Badge variant="default">Online</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Database</span>
                <Badge variant="default">Connected</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Workers</span>
                <Badge variant="default">Running</Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
