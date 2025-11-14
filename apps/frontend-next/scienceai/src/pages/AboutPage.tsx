import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { Brain, Target, Users } from "lucide-react";

const AboutPage = () => {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="flex-grow">
        <div className="container mx-auto px-4 py-12">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              About ScienceAI.news
            </h1>
            <p className="text-xl text-muted-foreground mb-12">
              Your trusted source for cutting-edge science and technology news
            </p>

            <div className="prose prose-lg max-w-none mb-16">
              <p className="text-foreground leading-relaxed mb-6 text-justify">
                ScienceAI is a digital publication dedicated to explaining with clarity and accuracy the scientific and technological advances that are shaping the future. We cover artificial intelligence, robotics, space exploration and data science with a focus on context, impact and precision.
              </p>
              <p className="text-foreground leading-relaxed mb-6 text-justify">
                Founded in 2025, ScienceAI.news was created to make science and technology accessible without simplification that distorts the facts or sensationalism. Our team of science journalists and researchers works to investigate, verify and select the stories that matter most.
              </p>
              <p className="text-foreground leading-relaxed mb-6 text-justify">
                We believe knowledge should be accessible to everyone and that a well informed society makes better decisions.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-16">
              <div className="bg-card rounded-xl p-6 shadow-card">
                <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center mb-4">
                  <Brain className="h-6 w-6 text-primary" />
                </div>
                <h3 className="text-xl font-bold mb-2">Expert Coverage</h3>
                <p className="text-muted-foreground text-justify">
                  Our journalists have deep expertise in their fields, ensuring
                  accurate and insightful reporting.
                </p>
              </div>

              <div className="bg-card rounded-xl p-6 shadow-card">
                <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center mb-4">
                  <Target className="h-6 w-6 text-primary" />
                </div>
                <h3 className="text-xl font-bold mb-2">Focused Content</h3>
                <p className="text-muted-foreground text-justify">
                  We concentrate on the most important developments that shape
                  our future.
                </p>
              </div>

              <div className="bg-card rounded-xl p-6 shadow-card">
                <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center mb-4">
                  <Users className="h-6 w-6 text-primary" />
                </div>
                <h3 className="text-xl font-bold mb-2">Community First</h3>
                <p className="text-muted-foreground text-justify">
                  We foster a community of curious minds passionate about
                  science and innovation.
                </p>
              </div>
            </div>

            <div className="bg-gradient-to-br from-primary/10 to-primary/5 rounded-xl p-8 border border-primary/20">
              <h2 className="text-2xl font-bold mb-4">Our Mission</h2>
              <p className="text-foreground leading-relaxed text-justify">
                To democratize access to scientific knowledge and inspire the
                next generation of innovators, researchers, and informed citizens.
                We strive to bridge the gap between complex research and public
                understanding, making science exciting and accessible for
                everyone.
              </p>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default AboutPage;
