import { Link, useLocation } from "react-router-dom";
import { 
  LayoutDashboard, 
  FileCode, 
  Newspaper, 
  FileText,
  Menu,
  X,
  Settings
} from "lucide-react";
import { useState } from "react";

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const location = useLocation();

  const menuItems = [
    { path: "/", icon: LayoutDashboard, label: "Dashboard" },
    { path: "/pages", icon: FileCode, label: "Pages" },
    { path: "/sources", icon: Newspaper, label: "Sources" },
    { path: "/logs", icon: FileText, label: "Logs" },
  ];

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className={`bg-card border-r border-border transition-all duration-300 ${
        sidebarOpen ? "w-64" : "w-20"
      }`}>
        <div className="p-4 flex items-center justify-between border-b border-border">
          <h1 className={`font-bold text-xl bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent animate-gradient ${
            sidebarOpen ? "block" : "hidden"
          }`}>
            News System
          </h1>
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="p-2 hover:bg-accent rounded-lg transition-colors"
          >
            {sidebarOpen ? <X size={20} /> : <Menu size={20} />}
          </button>
        </div>

        <nav className="mt-2 p-2">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = location.pathname === item.path;
            
            return (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200 ${
                  isActive 
                    ? "bg-primary text-primary-foreground shadow-sm" 
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                }`}
              >
                <Icon size={20} />
                <span className={`font-medium ${sidebarOpen ? "block" : "hidden"}`}>
                  {item.label}
                </span>
              </Link>
            );
          })}
        </nav>

        <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-border">
          <button className="flex items-center gap-3 px-4 py-3 rounded-lg text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-all w-full">
            <Settings size={20} />
            <span className={`font-medium ${sidebarOpen ? "block" : "hidden"}`}>
              Settings
            </span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  );
}
