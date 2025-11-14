import { useState, FormEvent } from "react";
import { Link } from "react-router-dom";
import { TrendingUp, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";

interface Article {
  id: string;
  slug: string;
  title: string;
  category: string;
  excerpt: string;
  content: string;
  date: string;
  author: string;
  readTime: number;
  imageCategories?: string[];
  image?: string; // Image path selected by server (using AIResearch logic)
}

interface SidebarProps {
  articles: Article[];
}

export const Sidebar = ({ articles }: SidebarProps) => {
  const mostRead = articles.slice(0, 5);
  const [email, setEmail] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubscribe = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    
    // Validar email básico
    if (!email || !email.includes('@')) {
      toast.error("Please enter a valid email address");
      return;
    }
    
    setIsSubmitting(true);
    
    try {
      const response = await fetch('/api/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email }),
      });
      
      const data = await response.json();
      
      if (response.ok) {
        if (data.alreadySubscribed) {
          toast.info("This email is already subscribed!");
        } else {
          toast.success("Successfully subscribed! Check your inbox for confirmation.");
        }
        setEmail(""); // Limpar campo após sucesso
      } else {
        toast.error(data.error || "Failed to subscribe. Please try again.");
      }
    } catch (error) {
      console.error('Error subscribing:', error);
      toast.error("Failed to subscribe. Please try again later.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <aside className="space-y-8">
      {/* Most Read */}
      <div className="bg-card rounded-xl p-6 shadow-card">
        <div className="flex items-center mb-4">
          <Eye className="h-5 w-5 text-primary mr-2" />
          <h3 className="text-lg font-bold">Most Read Today</h3>
        </div>
        <div className="space-y-4">
          {mostRead.map((article, index) => (
            <Link
              key={article.id}
              to={`/article/${article.slug}`}
              className="flex items-start space-x-3 group"
            >
              <span className="flex-shrink-0 w-8 h-8 bg-muted rounded-full flex items-center justify-center text-sm font-bold text-primary">
                {index + 1}
              </span>
              <div className="flex-1">
                <h4 className="text-sm font-medium group-hover:text-primary transition-smooth line-clamp-2">
                  {article.title}
                </h4>
                <p className="text-xs text-muted-foreground mt-1">
                  {article.readTime} min read
                </p>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Trending Topics */}
      <div className="bg-card rounded-xl p-6 shadow-card">
        <div className="flex items-center mb-4">
          <TrendingUp className="h-5 w-5 text-primary mr-2" />
          <h3 className="text-lg font-bold">Trending Topics</h3>
        </div>
        <div className="flex flex-wrap gap-2">
          {[
            "Machine Learning",
            "Quantum Computing",
            "Gene Therapy",
            "Mars Mission",
            "Neural Networks",
            "Climate Tech",
            "Biotech",
            "Space Exploration",
          ].map((topic) => (
            <span
              key={topic}
              className="px-3 py-1 bg-muted text-foreground text-sm rounded-full hover:bg-primary hover:text-primary-foreground transition-smooth cursor-pointer"
            >
              {topic}
            </span>
          ))}
        </div>
      </div>

      {/* Newsletter */}
      <div className="bg-gradient-to-br from-primary/10 to-primary/5 rounded-xl p-6 border border-primary/20">
        <h3 className="text-lg font-bold mb-2">Stay Updated</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Get the latest science news delivered to your inbox every week.
        </p>
        <form className="space-y-3" onSubmit={handleSubscribe}>
          <Input 
            type="email" 
            placeholder="Enter your email" 
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            disabled={isSubmitting}
            required
          />
          <Button 
            type="submit" 
            className="w-full gradient-primary" 
            disabled={isSubmitting}
          >
            {isSubmitting ? "Subscribing..." : "Subscribe"}
          </Button>
        </form>
      </div>
    </aside>
  );
};