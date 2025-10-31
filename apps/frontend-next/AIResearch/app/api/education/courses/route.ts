import { NextResponse } from 'next/server';

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

interface CoursesResponse {
  courses: Course[];
  total: number;
  platforms: string[];
  categories: string[];
}

// Cache para evitar múltiplas chamadas à API
let coursesCache: Course[] | null = null;
let cacheTimestamp: number = 0;
const CACHE_DURATION = 3600000; // 1 hora em milissegundos

// Dados manuais como fallback e base
const manualCourses: Course[] = [
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

/**
 * Busca cursos do edX (API limitada, podemos fazer scraping ou usar dados manuais)
 * NOTA: edX não tem API pública completa, mas podemos buscar informações básicas
 * FILTRADO: Apenas cursos de tecnologia (Computer Science, AI, Data Science, Programming)
 */
async function fetchEdXCourses(): Promise<Course[]> {
  try {
    // Tentar buscar dados do edX filtrados por categorias de tecnologia
    // Categorias: computer-science, artificial-intelligence, data-science, programming
    // Por enquanto, retornar array vazio e usar dados manuais filtrados
    // TODO: Implementar quando tivermos acesso à API do edX
    // Quando implementar, usar filtros: ?subject=Computer+Science&subject=Artificial+Intelligence
    
    // Filtrar manualCourses por categorias de tecnologia para simular filtragem
    const techCategories = ['Machine Learning', 'Introdução à IA', 'NLP', 'Computer Vision', 'Certificação'];
    return manualCourses.filter(c => techCategories.includes(c.category));
  } catch (error) {
    console.error('Error fetching edX courses:', error);
    return [];
  }
}

/**
 * Busca cursos do MIT OpenCourseWare (possui dados estruturados)
 * FILTRADO: Apenas cursos do departamento de Electrical Engineering and Computer Science
 */
async function fetchMITCourses(): Promise<Course[]> {
  try {
    // MIT OpenCourseWare tem dados JSON disponíveis por departamento
    // Departamento: Electrical-Engineering-and-Computer-Science
    // Exemplo: https://ocw.mit.edu/courses/electrical-engineering-and-computer-science/
    // TODO: Implementar busca filtrada por departamento de tecnologia
    
    // Por enquanto, retornar array vazio (quando implementar, buscar apenas CS/EE courses)
    return [];
  } catch (error) {
    console.error('Error fetching MIT courses:', error);
    return [];
  }
}

/**
 * Busca todos os cursos de todas as fontes
 */
async function fetchAllCourses(): Promise<Course[]> {
  // Verificar cache primeiro
  const now = Date.now();
  if (coursesCache && (now - cacheTimestamp) < CACHE_DURATION) {
    return coursesCache;
  }

  try {
    // Buscar de múltiplas fontes
    const [edxCourses, mitCourses] = await Promise.all([
      fetchEdXCourses(),
      fetchMITCourses(),
    ]);

    // Combinar todos os cursos (já filtrados para tecnologia)
    // Filtrar manualCourses para garantir que são apenas de tecnologia
    const techCategories = [
      'Machine Learning', 
      'Introdução à IA', 
      'NLP', 
      'Computer Vision', 
      'Certificação',
      'Artificial Intelligence',
      'Data Science',
      'Programming'
    ];
    
    const filteredManualCourses = manualCourses.filter(c => 
      techCategories.some(cat => 
        c.category.toLowerCase().includes(cat.toLowerCase()) ||
        c.title.toLowerCase().includes('ai') ||
        c.title.toLowerCase().includes('machine learning') ||
        c.title.toLowerCase().includes('deep learning') ||
        c.title.toLowerCase().includes('computer science') ||
        c.title.toLowerCase().includes('programming')
      )
    );
    
    const allCourses = [
      ...filteredManualCourses,
      ...edxCourses,
      ...mitCourses,
    ];

    // Atualizar cache
    coursesCache = allCourses;
    cacheTimestamp = now;

    return allCourses;
  } catch (error) {
    console.error('Error fetching all courses:', error);
    // Retornar dados manuais como fallback
    return manualCourses;
  }
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const category = searchParams.get('category');
  const platform = searchParams.get('platform');
  const language = searchParams.get('language');
  const search = searchParams.get('search');

  try {
    // Buscar todos os cursos
    let allCourses = await fetchAllCourses();

    // Filtrar por categoria
    if (category && category !== 'all') {
      allCourses = allCourses.filter(c => c.category === category);
    }

    // Filtrar por plataforma
    if (platform) {
      allCourses = allCourses.filter(c => c.platform.toLowerCase() === platform.toLowerCase());
    }

    // Filtrar por idioma
    if (language) {
      allCourses = allCourses.filter(c => c.language?.toLowerCase() === language.toLowerCase());
    }

    // Filtrar por busca (título, descrição, instrutor)
    if (search) {
      const searchLower = search.toLowerCase();
      allCourses = allCourses.filter(c =>
        c.title.toLowerCase().includes(searchLower) ||
        c.description.toLowerCase().includes(searchLower) ||
        c.instructor.toLowerCase().includes(searchLower) ||
        c.platform.toLowerCase().includes(searchLower)
      );
    }

    // Extrair plataformas e categorias únicas
    const platforms = Array.from(new Set(allCourses.map(c => c.platform)));
    const categories = Array.from(new Set(allCourses.map(c => c.category)));

    const response: CoursesResponse = {
      courses: allCourses,
      total: allCourses.length,
      platforms,
      categories,
    };

    return NextResponse.json(response, {
      headers: {
        'Cache-Control': 'public, s-maxage=3600, stale-while-revalidate=86400',
      },
    });
  } catch (error) {
    console.error('Error in courses API:', error);
    
    // Fallback para dados manuais
    const platforms = Array.from(new Set(manualCourses.map(c => c.platform)));
    const categories = Array.from(new Set(manualCourses.map(c => c.category)));
    
    return NextResponse.json({
      courses: manualCourses,
      total: manualCourses.length,
      platforms,
      categories,
      error: 'Some data sources may be unavailable',
    });
  }
}
