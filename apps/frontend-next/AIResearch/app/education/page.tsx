import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Clock, BookOpen, ExternalLink, CheckCircle } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

// Mock data - will be replaced with actual API calls
// Note: Links de afiliados devem ser substituídos pelos links reais
const courses = [
  {
    id: "1",
    title: "Deep Learning Specialization",
    platform: "Coursera",
    instructor: "Andrew Ng - Stanford",
    category: "Machine Learning",
    duration: "5 meses",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.8,
    students: 500000,
    description: "A especialização completa de Deep Learning de Andrew Ng cobrindo CNN, RNN, Transformers e mais. Curso obrigatório para qualquer profissional de IA.",
    url: "https://www.coursera.org/specializations/deep-learning",
    affiliate: true,
  },
  {
    id: "2",
    title: "Machine Learning de Stanford",
    platform: "Coursera",
    instructor: "Andrew Ng",
    category: "Machine Learning",
    duration: "11 semanas",
    level: "Iniciante",
    price: "Grátis (certificado pago)",
    rating: 4.9,
    students: 3000000,
    description: "O curso mais popular de Machine Learning no mundo. Fundamentos de ML, aprendizado supervisionado e não-supervisionado.",
    url: "https://www.coursera.org/learn/machine-learning",
    affiliate: true,
  },
  {
    id: "3",
    title: "CS50's Introduction to AI with Python",
    platform: "edX",
    instructor: "Harvard University",
    category: "Introdução à IA",
    duration: "7 semanas",
    level: "Iniciante",
    price: "Grátis",
    rating: 4.9,
    students: 500000,
    description: "Curso de Harvard sobre fundamentos de IA, search algorithms, machine learning, neural networks e muito mais",
    url: "https://www.edx.org/learn/artificial-intelligence/harvard-university-cs50-s-introduction-to-artificial-intelligence-with-python",
    affiliate: true,
  },
  {
    id: "4",
    title: "Natural Language Processing",
    platform: "Coursera",
    instructor: "Stanford & DeepLearning.ai",
    category: "NLP",
    duration: "4 semanas",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.7,
    students: 150000,
    description: "Aprenda processamento de linguagem natural, sentiment analysis, word embeddings, Transformers e LLMs",
    url: "https://www.coursera.org/learn/classification-vector-spaces-in-nlp",
    affiliate: true,
  },
  {
    id: "5",
    title: "Stanford CS231n: Computer Vision",
    platform: "YouTube / Stanford",
    instructor: "Stanford University",
    category: "Computer Vision",
    duration: "10 horas",
    level: "Avançado",
    price: "Grátis",
    rating: 4.9,
    students: 1000000,
    description: "Curso completo de Stanford sobre visão computacional. CNN, object detection, segmentation, GANs.",
    url: "https://www.youtube.com/playlist?list=PL3FW7Lu3i5JvHM8ljYj-zLfQRF3KO8WR-",
    affiliate: false,
  },
  {
    id: "6",
    title: "Practical Deep Learning for Coders",
    platform: "fast.ai",
    instructor: "Jeremy Howard",
    category: "Machine Learning",
    duration: "8 semanas",
    level: "Todos os níveis",
    price: "Grátis",
    rating: 4.8,
    students: 200000,
    description: "Aprenda deep learning de forma prática e com foco em código. Curso reconhecido pela comunidade científica.",
    url: "https://course.fast.ai",
    affiliate: false,
  },
  {
    id: "7",
    title: "Machine Learning A-Z",
    platform: "Udemy",
    instructor: "Kirill Eremenko",
    category: "Machine Learning",
    duration: "40 horas",
    level: "Iniciante",
    price: "Pago (sempre em promoção)",
    rating: 4.7,
    students: 800000,
    description: "Curso completo de Machine Learning do zero. Data preprocessing, regression, classification, clustering e mais.",
    url: "https://www.udemy.com/course/machinelearning",
    affiliate: true,
  },
  {
    id: "8",
    title: "Deep Learning e Computer Vision",
    platform: "Udemy",
    instructor: "Jose Portilla",
    category: "Computer Vision",
    duration: "50 horas",
    level: "Intermediário",
    price: "Pago (sempre em promoção)",
    rating: 4.6,
    students: 400000,
    description: "Curso completo sobre deep learning aplicado a visão computacional com TensorFlow e Keras.",
    url: "https://www.udemy.com/course/computer-vision-a-z",
    affiliate: true,
  },
  {
    id: "9",
    title: "TensorFlow Developer Certificate",
    platform: "Coursera",
    instructor: "TensorFlow Team",
    category: "Certificação",
    duration: "4 meses",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.8,
    students: 50000,
    description: "Preparação para o TensorFlow Certification exam. Aprenda building and training neural networks.",
    url: "https://www.coursera.org/professional-certificates/tensorflow-in-practice",
    affiliate: true,
  },
];

export default function EducationPage() {
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        {/* Hero Section */}
        <div className="relative bg-gradient-to-br from-primary/5 via-background to-background py-12">
          <div className="container mx-auto px-4">
            <div className="max-w-4xl mx-auto text-center">
              <h1 className="text-4xl md:text-5xl font-bold mb-4 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
                Educação em IA
              </h1>
              <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-6">
                Descubra cursos nas melhores instituições de ensino do mundo
              </p>
              
              {/* Affiliate Disclosure */}
              <div className="max-w-2xl mx-auto">
                <p className="text-sm text-muted-foreground bg-muted/50 px-4 py-3 rounded-lg border border-border">
                  ℹ️ <strong>Aviso Legal:</strong> Alguns links podem ser afiliados. Isso significa que podemos receber uma pequena comissão se você fizer uma compra através desses links, sem custo adicional para você. Isso nos ajuda a manter o site gratuitamente.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Courses Grid */}
        <section className="container mx-auto px-4 py-12">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {courses.map((course) => (
              <Card key={course.id} className="group relative overflow-hidden hover:border-primary/50 transition-all duration-300 hover-lift h-full flex flex-col">
                <CardHeader className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                      {course.category}
                    </span>
                    <div className="flex items-center gap-1">
                      <CheckCircle className="h-4 w-4 text-primary" />
                      <span className="text-xs font-medium">{course.rating}</span>
                    </div>
                  </div>
                  <CardTitle className="line-clamp-2 group-hover:text-primary transition-colors">
                    {course.title}
                  </CardTitle>
                  <CardDescription className="line-clamp-2 mt-2">
                    {course.description}
                  </CardDescription>
                </CardHeader>

                <CardContent className="flex-1 flex flex-col">
                  <div className="space-y-3 mb-4 text-sm text-muted-foreground">
                    <div className="flex items-center gap-2">
                      <BookOpen className="h-4 w-4" />
                      <span>{course.instructor}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Clock className="h-4 w-4" />
                      <span>{course.duration}</span>
                      <span>•</span>
                      <span>{course.level}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{course.students.toLocaleString()} alunos</span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between pt-4 border-t border-border mt-auto">
                    <div className="flex items-center gap-2 flex-wrap">
                      {course.price === "Grátis" ? (
                        <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                          Gratuito
                        </span>
                      ) : course.price.includes("Grátis") ? (
                        <>
                          <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                            Gratuito
                          </span>
                          <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                            Certificado
                          </span>
                        </>
                      ) : course.price.includes("Pago") ? (
                        <span className="px-3 py-1 text-xs font-semibold rounded-full bg-orange-500/10 text-orange-600 border border-orange-500/20">
                          Pago
                        </span>
                      ) : (
                        <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                          {course.price}
                        </span>
                      )}
                      {course.affiliate && (
                        <span className="px-2 py-1 text-xs font-medium rounded bg-muted text-muted-foreground">
                          Afiliado
                        </span>
                      )}
                    </div>
                    <Button variant="outline" size="sm" className="gap-2">
                      Acessar
                      <ExternalLink className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
}

