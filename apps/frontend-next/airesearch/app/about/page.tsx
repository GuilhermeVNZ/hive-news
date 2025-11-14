import Header from "@/components/Header";
import Footer from "@/components/Footer";

const focusAreas = ["Machine learning and applied AI", "Responsible innovation and policy", "Tools for builders and researchers", "Education resources for teams of all sizes"];

const values = ["Clarity first: we translate complex updates into language everyone can use", "Evidence-based: every highlight points back to trusted sources", "Forward-looking: we connect daily releases to long-term trends", "Community-minded: we share knowledge that helps people build together"];

export default function AboutPage() {
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        <section className="relative bg-gradient-to-br from-primary/5 via-background to-background py-16">
          <div className="container mx-auto px-4">
            <div className="max-w-3xl mx-auto text-center">
              <h1 className="text-4xl md:text-5xl font-bold bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
                About AIResearch
              </h1>
              <p className="mt-6 text-lg text-muted-foreground">
                AIResearch is a news and insights destination focused on the people, ideas, and breakthroughs shaping artificial intelligence. We highlight the stories that move the field forward and share the context readers need to understand why each development matters.
              </p>
            </div>
          </div>
        </section>

        <section className="container mx-auto px-4 py-16">
          <div className="grid gap-10 lg:grid-cols-2">
            <div className="rounded-3xl border border-primary/20 bg-card/80 p-8 shadow-lg backdrop-blur">
              <h2 className="text-2xl font-semibold">Our Perspective</h2>
              <p className="mt-4 text-muted-foreground leading-relaxed">
                We follow AI progress with curiosity and optimism. That means celebrating new discoveries, examining their impact, and giving our audience a balanced view of what comes next. AIResearch connects the dots between academia, industry, and policy so readers can stay informed without feeling overwhelmed.
              </p>
            </div>

            <div className="rounded-3xl border border-border bg-background/80 p-8 shadow-sm">
              <h2 className="text-2xl font-semibold">What We Cover</h2>
              <ul className="mt-6 space-y-3 text-muted-foreground">
                {focusAreas.map((area) => (
                  <li key={area} className="rounded-2xl border border-border/60 bg-card/60 p-4 text-sm leading-relaxed">
                    {area}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>

        <section className="bg-card/50 py-16">
          <div className="container mx-auto px-4">
            <div className="max-w-4xl mx-auto text-center">
              <h2 className="text-3xl font-semibold">The Values Behind Our Stories</h2>
              <p className="mt-4 text-muted-foreground">
                Simple ideas guide our work. They help us choose the stories we feature and the voice we use to tell them.
              </p>
            </div>
            <div className="mt-10 grid gap-6 md:grid-cols-2">
              {values.map((value) => (
                <div key={value} className="rounded-3xl border border-border bg-background/90 p-6 shadow-sm text-sm leading-relaxed text-muted-foreground">
                  {value}
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="container mx-auto px-4 py-16">
          <div className="rounded-3xl border border-primary/30 bg-primary/5 p-10 text-center shadow-lg">
            <h2 className="text-3xl font-semibold">Stay Connected</h2>
            <p className="mt-4 text-muted-foreground">
              We love hearing from readers, educators, researchers, and teams building with AI. Share a story idea, suggest a topic, or collaborate with us.
            </p>
            <p className="mt-6 text-sm text-muted-foreground">
              Reach out at <a className="text-primary hover:underline" href="mailto:contact@hive-hub.ai">contact@hive-hub.ai</a>
            </p>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
}

