import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

const Hero = () => {
  return (
    <section className="relative bg-gradient-to-br from-primary/5 via-background to-background py-8 md:py-12">
      <div className="container mx-auto px-4">
        <div className="max-w-4xl mx-auto text-center animate-fade-in-up">
          <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold text-foreground mb-10">
            Notícias Científicas sobre
            <span className="text-primary block mt-2">Inteligência Artificial</span>
          </h1>

          <div className="flex flex-col sm:flex-row gap-3 max-w-2xl mx-auto">
            <div className="relative flex-1">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
              <Input
                placeholder="Buscar artigos, tópicos ou pesquisadores..."
                className="pl-12 h-12 text-base border-border bg-card"
              />
            </div>
            <Button className="h-12 px-8 bg-primary hover:bg-primary/90 text-primary-foreground">
              Buscar
            </Button>
          </div>

          <div className="flex flex-wrap gap-2 justify-center mt-8">
            {["Machine Learning", "LLMs", "Computer Vision", "Robótica", "NLP"].map((tag) => (
              <button
                key={tag}
                className="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-primary hover:bg-primary/5 rounded-full border border-border transition-all hover:border-primary"
              >
                {tag}
              </button>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};

export default Hero;
