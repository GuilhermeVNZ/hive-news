"use client";

import { useState, useMemo, useEffect } from "react";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Clock, BookOpen, ExternalLink, CheckCircle, Search } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface Course {
  id: string;
  title: string;
  platform: string;
  instructor: string;
  institution?: string;
  category: string;
  duration: string;
  level: string;
  price: string;
  rating?: number;
  description: string;
  url: string;
  language?: string;
  image_url?: string;
  affiliate: boolean;
  certificate_available: boolean;
  free: boolean;
}

// Fallback mock data - usado apenas se API falhar
const fallbackCourses: Course[] = [
  {
    id: "1",
    title: "Deep Learning Specialization",
    platform: "Coursera",
    instructor: "Andrew Ng - Stanford",
    institution: "Stanford University",
    category: "Machine Learning",
    duration: "5 meses",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.8,
    description: "A especialização completa de Deep Learning de Andrew Ng cobrindo CNN, RNN, Transformers e mais. Curso obrigatório para qualquer profissional de IA.",
    url: "https://www.coursera.org/specializations/deep-learning",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
  {
    id: "2",
    title: "Machine Learning de Stanford",
    platform: "Coursera",
    instructor: "Andrew Ng",
    institution: "Stanford University",
    category: "Machine Learning",
    duration: "11 semanas",
    level: "Iniciante",
    price: "Grátis (certificado pago)",
    rating: 4.9,
    description: "O curso mais popular de Machine Learning no mundo. Fundamentos de ML, aprendizado supervisionado e não-supervisionado.",
    url: "https://www.coursera.org/learn/machine-learning",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
  {
    id: "3",
    title: "CS50's Introduction to AI with Python",
    platform: "edX",
    instructor: "Harvard University",
    institution: "Harvard University",
    category: "Introdução à IA",
    duration: "7 semanas",
    level: "Iniciante",
    price: "Grátis",
    rating: 4.9,
    description: "Curso de Harvard sobre fundamentos de IA, search algorithms, machine learning, neural networks e muito mais",
    url: "https://www.edx.org/learn/artificial-intelligence/harvard-university-cs50-s-introduction-to-artificial-intelligence-with-python",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
  {
    id: "4",
    title: "Natural Language Processing",
    platform: "Coursera",
    instructor: "Stanford & DeepLearning.ai",
    institution: "Stanford University",
    category: "NLP",
    duration: "4 semanas",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.7,
    description: "Aprenda processamento de linguagem natural, sentiment analysis, word embeddings, Transformers e LLMs",
    url: "https://www.coursera.org/learn/classification-vector-spaces-in-nlp",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
  {
    id: "5",
    title: "Stanford CS231n: Computer Vision",
    platform: "YouTube / Stanford",
    instructor: "Stanford University",
    institution: "Stanford University",
    category: "Computer Vision",
    duration: "10 semanas",
    level: "Avançado",
    price: "Grátis",
    rating: 4.9,
    description: "Curso completo de Stanford sobre visão computacional. CNN, object detection, segmentation, GANs.",
    url: "https://www.youtube.com/playlist?list=PL3FW7Lu3i5JvHM8ljYj-zLfQRF3KO8WR-",
    language: "en",
    affiliate: false,
    certificate_available: false,
    free: true,
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
    description: "Aprenda deep learning de forma prática e com foco em código. Curso reconhecido pela comunidade científica.",
    url: "https://course.fast.ai",
    language: "en",
    affiliate: false,
    certificate_available: false,
    free: true,
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
    description: "Curso completo de Machine Learning do zero. Data preprocessing, regression, classification, clustering e mais.",
    url: "https://www.udemy.com/course/machinelearning",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: false,
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
    description: "Curso completo sobre deep learning aplicado a visão computacional com TensorFlow e Keras.",
    url: "https://www.udemy.com/course/computer-vision-a-z",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: false,
  },
  {
    id: "9",
    title: "TensorFlow Developer Certificate",
    platform: "Coursera",
    instructor: "TensorFlow Team",
    institution: "Google",
    category: "Certificação",
    duration: "4 meses",
    level: "Intermediário",
    price: "Grátis (certificado pago)",
    rating: 4.8,
    description: "Preparação para o TensorFlow Certification exam. Aprenda building and training neural networks.",
    url: "https://www.coursera.org/professional-certificates/tensorflow-in-practice",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
];

export default function EducationPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [courses, setCourses] = useState<Course[]>(fallbackCourses);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Buscar cursos da API ao montar o componente
  useEffect(() => {
    async function fetchCourses() {
      try {
        setLoading(true);
        // Buscar apenas cursos de tecnologia
        const response = await fetch('/api/education/courses?category=technology');
        const data = await response.json();
        
        // Filtrar cursos para garantir que são apenas de tecnologia
        const techCategories = [
          'Machine Learning', 
          'Introdução à IA', 
          'NLP', 
          'Computer Vision', 
          'Certificação',
          'Artificial Intelligence',
          'Data Science',
          'Programming',
          'Computer Science'
        ];
        
        let filteredCourses = [];
        if (data.courses && data.courses.length > 0) {
          // Filtrar cursos por categoria de tecnologia
          filteredCourses = data.courses.filter((c: Course) => 
            techCategories.some(cat => 
              c.category.toLowerCase().includes(cat.toLowerCase()) ||
              c.title.toLowerCase().includes('ai') ||
              c.title.toLowerCase().includes('machine learning') ||
              c.title.toLowerCase().includes('deep learning') ||
              c.title.toLowerCase().includes('computer science') ||
              c.title.toLowerCase().includes('programming') ||
              c.title.toLowerCase().includes('artificial intelligence')
            )
          );
        }
        
        if (filteredCourses.length > 0) {
          setCourses(filteredCourses);
          setError(null);
        } else {
          // Usar fallback filtrado se API retornar vazio
          console.log('No courses from API, using filtered fallback data');
          const filteredFallback = fallbackCourses.filter(c => 
            techCategories.some(cat => 
              c.category.toLowerCase().includes(cat.toLowerCase()) ||
              c.title.toLowerCase().includes('ai') ||
              c.title.toLowerCase().includes('machine learning') ||
              c.title.toLowerCase().includes('deep learning') ||
              c.title.toLowerCase().includes('computer science')
            )
          );
          setCourses(filteredFallback);
        }
      } catch (err) {
        console.error('Error fetching courses:', err);
        setError('Failed to load courses');
        // Usar dados de fallback filtrados em caso de erro
        const techCategories = [
          'Machine Learning', 
          'Introdução à IA', 
          'NLP', 
          'Computer Vision', 
          'Certificação'
        ];
        const filteredFallback = fallbackCourses.filter(c => 
          techCategories.some(cat => 
            c.category.toLowerCase().includes(cat.toLowerCase()) ||
            c.title.toLowerCase().includes('ai') ||
            c.title.toLowerCase().includes('machine learning')
          )
        );
        setCourses(filteredFallback);
      } finally {
        setLoading(false);
      }
    }
    
    fetchCourses();
  }, []);

  // Unique categories from courses
  const categories = useMemo(() => {
    const uniqueCats = Array.from(new Set(courses.map(c => c.category)));
    return ["all", ...uniqueCats];
  }, [courses]);

  // Filter courses based on search and category
  const filteredCourses = useMemo(() => {
    return courses.filter((course) => {
      const matchesSearch = 
        course.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        course.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        course.platform.toLowerCase().includes(searchQuery.toLowerCase()) ||
        course.instructor.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesCategory = selectedCategory === "all" || course.category === selectedCategory;
      
      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory, courses]);

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        {/* Hero Section */}
        <div className="relative bg-gradient-to-br from-primary/5 via-background to-background py-12">
          <div className="container mx-auto px-4">
            <div className="max-w-4xl mx-auto">
              <h1 className="text-4xl md:text-5xl font-bold mb-4 text-center bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
                Educação em IA
              </h1>
              <p className="text-xl text-muted-foreground max-w-2xl mx-auto text-center mb-8">
                Descubra cursos nas melhores instituições de ensino do mundo
              </p>
              
              {/* Search Bar */}
              <div className="max-w-2xl mx-auto">
                <div className="relative">
                  <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                  <Input
                    type="text"
                    placeholder="Pesquisar por curso, instituição, instrutor ou tema..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-12 h-14 text-base border-2 border-border bg-background focus:border-primary transition-all"
                  />
                </div>
              </div>

              {/* Category Filter */}
              <div className="flex flex-wrap gap-2 justify-center mt-6">
                {categories.map((category) => (
                  <button
                    key={category}
                    onClick={() => setSelectedCategory(category)}
                    className={`px-4 py-2 rounded-full text-sm font-medium transition-all ${
                      selectedCategory === category
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground hover:bg-accent"
                    }`}
                  >
                    {category === "all" ? "Todos" : category}
                  </button>
                ))}
              </div>

              {/* Results count */}
              <p className="text-center text-sm text-muted-foreground mt-6">
                {filteredCourses.length} {filteredCourses.length === 1 ? "curso encontrado" : "cursos encontrados"}
              </p>
            </div>
          </div>
        </div>

        {/* Courses Grid */}
        <section className="container mx-auto px-4 py-12">
          {loading ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">Carregando cursos...</p>
            </div>
          ) : error ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">Erro ao carregar cursos</p>
              <p className="text-sm text-muted-foreground mt-2">Usando dados em cache</p>
            </div>
          ) : filteredCourses.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">Nenhum curso encontrado</p>
              <p className="text-sm text-muted-foreground mt-2">Tente alterar sua busca ou filtro</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {filteredCourses.map((course) => (
              <Card key={course.id} className="group relative overflow-hidden hover:border-primary/50 transition-all duration-300 hover-lift h-full flex flex-col">
                <CardHeader className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                      {course.category}
                    </span>
                    {course.rating && (
                      <div className="flex items-center gap-1">
                        <CheckCircle className="h-4 w-4 text-primary" />
                        <span className="text-xs font-medium">{course.rating.toFixed(1)}</span>
                      </div>
                    )}
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
                  </div>

                    <div className="flex items-center justify-between pt-4 border-t border-border mt-auto">
                    <div className="flex items-center gap-2 flex-wrap">
                      {course.free ? (
                        <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                          Gratuito
                        </span>
                      ) : course.price.includes("Grátis") ? (
                        <>
                          <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                            Gratuito
                          </span>
                          {course.certificate_available && (
                            <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                              Certificado
                            </span>
                          )}
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
                    <Button 
                      variant="outline" 
                      size="sm" 
                      className="gap-2"
                      onClick={() => window.open(course.url, '_blank')}
                    >
                      Acessar
                      <ExternalLink className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
              ))}
            </div>
          )}
        </section>
      </main>
      <Footer />
    </div>
  );
}

