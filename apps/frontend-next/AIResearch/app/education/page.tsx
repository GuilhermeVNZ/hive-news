"use client";

import { useState, useMemo, useEffect } from "react";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import {
  Clock,
  BookOpen,
  ExternalLink,
  CheckCircle,
  Search,
} from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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

// Fallback mock data - used only if the API fails
const fallbackCourses: Course[] = [
  {
    id: "1",
    title: "Deep Learning Specialization",
    platform: "Coursera",
    instructor: "Andrew Ng - Stanford",
    institution: "Stanford University",
    category: "Machine Learning",
    duration: "5 months",
    level: "Intermediate",
    price: "Free (paid certificate)",
    rating: 4.8,
    description:
      "Andrew Ng's complete Deep Learning specialization covering CNNs, RNNs, Transformers, and more. A must-take course for any AI professional.",
    url: "https://www.coursera.org/specializations/deep-learning",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: true,
  },
  {
    id: "2",
    title: "Stanford Machine Learning",
    platform: "Coursera",
    instructor: "Andrew Ng",
    institution: "Stanford University",
    category: "Machine Learning",
    duration: "11 weeks",
    level: "Beginner",
    price: "Free (paid certificate)",
    rating: 4.9,
    description:
      "The world's most popular Machine Learning course. Covers ML fundamentals, supervised, and unsupervised learning.",
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
    category: "AI Introduction",
    duration: "7 weeks",
    level: "Beginner",
    price: "Free",
    rating: 4.9,
    description:
      "Harvard course covering AI fundamentals, search algorithms, machine learning, neural networks, and more.",
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
    duration: "4 weeks",
    level: "Intermediate",
    price: "Free (paid certificate)",
    rating: 4.7,
    description:
      "Learn natural language processing, sentiment analysis, word embeddings, Transformers, and LLMs.",
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
    duration: "10 weeks",
    level: "Advanced",
    price: "Free",
    rating: 4.9,
    description:
      "Complete Stanford course on computer vision covering CNNs, object detection, segmentation, and GANs.",
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
    duration: "8 weeks",
    level: "All levels",
    price: "Free",
    rating: 4.8,
    description:
      "Learn deep learning with a practical, code-first approach. Highly regarded within the AI community.",
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
    duration: "40 hours",
    level: "Beginner",
    price: "Paid (often on sale)",
    rating: 4.7,
    description:
      "End-to-end Machine Learning course covering data preprocessing, regression, classification, clustering, and more.",
    url: "https://www.udemy.com/course/machinelearning",
    language: "en",
    affiliate: true,
    certificate_available: true,
    free: false,
  },
  {
    id: "8",
    title: "Deep Learning and Computer Vision",
    platform: "Udemy",
    instructor: "Jose Portilla",
    category: "Computer Vision",
    duration: "50 hours",
    level: "Intermediate",
    price: "Paid (often on sale)",
    rating: 4.6,
    description:
      "Comprehensive course on deep learning applied to computer vision with TensorFlow and Keras.",
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
    category: "Certification",
    duration: "4 months",
    level: "Intermediate",
    price: "Free (paid certificate)",
    rating: 4.8,
    description:
      "Preparation for the TensorFlow certification exam. Learn how to build and train neural networks.",
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

  // Fetch courses from the API when the component mounts
  useEffect(() => {
    async function fetchCourses() {
      try {
        setLoading(true);
        // Request only technology courses
        const response = await fetch(
          "/api/education/courses?category=technology",
        );
        const data = await response.json();

        // Filter courses to ensure they focus on technology topics
        const techCategories = [
          "Machine Learning",
          "AI Introduction",
          "NLP",
          "Computer Vision",
          "Certification",
          "Artificial Intelligence",
          "Data Science",
          "Programming",
          "Computer Science",
        ];

        let filteredCourses = [];
        if (data.courses && data.courses.length > 0) {
          // Filter courses by technology categories
          filteredCourses = data.courses.filter((c: Course) =>
            techCategories.some(
              (cat) =>
                c.category.toLowerCase().includes(cat.toLowerCase()) ||
                c.title.toLowerCase().includes("ai") ||
                c.title.toLowerCase().includes("machine learning") ||
                c.title.toLowerCase().includes("deep learning") ||
                c.title.toLowerCase().includes("computer science") ||
                c.title.toLowerCase().includes("programming") ||
                c.title.toLowerCase().includes("artificial intelligence"),
            ),
          );
        }

        if (filteredCourses.length > 0) {
          setCourses(filteredCourses);
          setError(null);
        } else {
          // Use filtered fallback data if the API returns nothing
          console.log("No courses from API, using filtered fallback data");
          const filteredFallback = fallbackCourses.filter((c) =>
            techCategories.some(
              (cat) =>
                c.category.toLowerCase().includes(cat.toLowerCase()) ||
                c.title.toLowerCase().includes("ai") ||
                c.title.toLowerCase().includes("machine learning") ||
                c.title.toLowerCase().includes("deep learning") ||
                c.title.toLowerCase().includes("computer science"),
            ),
          );
          setCourses(filteredFallback);
        }
      } catch (err) {
        console.error("Error fetching courses:", err);
        setError("Failed to load courses");
        // Use filtered fallback data if the request fails
        const techCategories = [
          "Machine Learning",
          "AI Introduction",
          "NLP",
          "Computer Vision",
          "Certification",
        ];
        const filteredFallback = fallbackCourses.filter((c) =>
          techCategories.some(
            (cat) =>
              c.category.toLowerCase().includes(cat.toLowerCase()) ||
              c.title.toLowerCase().includes("ai") ||
              c.title.toLowerCase().includes("machine learning"),
          ),
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
    const uniqueCats = Array.from(new Set(courses.map((c) => c.category)));
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

      const matchesCategory =
        selectedCategory === "all" || course.category === selectedCategory;

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
                AI Education
              </h1>
              <p className="text-xl text-muted-foreground max-w-2xl mx-auto text-center mb-8">
                Discover courses from the world&rsquo;s leading education
                institutions
              </p>

              {/* Search Bar */}
              <div className="max-w-2xl mx-auto">
                <div className="relative">
                  <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                  <Input
                    type="text"
                    placeholder="Search by course, institution, instructor, or topic..."
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
                    {category === "all" ? "All" : category}
                  </button>
                ))}
              </div>

              {/* Results count */}
              <p className="text-center text-sm text-muted-foreground mt-6">
                {filteredCourses.length}{" "}
                {filteredCourses.length === 1
                  ? "course found"
                  : "courses found"}
              </p>
            </div>
          </div>
        </div>

        {/* Courses Grid */}
        <section className="container mx-auto px-4 py-12">
          {loading ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">
                Loading courses...
              </p>
            </div>
          ) : error ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">
                Failed to load courses
              </p>
              <p className="text-sm text-muted-foreground mt-2">
                Using cached data
              </p>
            </div>
          ) : filteredCourses.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-xl text-muted-foreground">No courses found</p>
              <p className="text-sm text-muted-foreground mt-2">
                Try adjusting your search or filters
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {filteredCourses.map((course) => (
                <Card
                  key={course.id}
                  className="group relative overflow-hidden hover:border-primary/50 transition-all duration-300 hover-lift h-full flex flex-col"
                >
                  <CardHeader className="relative">
                    <div className="flex items-center justify-between mb-3">
                      <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                        {course.category}
                      </span>
                      {course.rating && (
                        <div className="flex items-center gap-1">
                          <CheckCircle className="h-4 w-4 text-primary" />
                          <span className="text-xs font-medium">
                            {course.rating.toFixed(1)}
                          </span>
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
                          <>
                            <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                              Free
                            </span>
                            {course.certificate_available && (
                              <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                                Certificate
                              </span>
                            )}
                          </>
                        ) : course.price.toLowerCase().includes("free") ? (
                          <>
                            <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/10 text-green-600 border border-green-500/20">
                              Free
                            </span>
                            {course.certificate_available && (
                              <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                                Certificate
                              </span>
                            )}
                          </>
                        ) : course.price.toLowerCase().includes("paid") ? (
                          <span className="px-3 py-1 text-xs font-semibold rounded-full bg-orange-500/10 text-orange-600 border border-orange-500/20">
                            Paid
                          </span>
                        ) : (
                          <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                            {course.price}
                          </span>
                        )}
                        {course.certificate_available &&
                          !course.price.toLowerCase().includes("free") &&
                          !course.free && (
                            <span className="px-3 py-1 text-xs font-semibold rounded-full bg-primary/10 text-primary border border-primary/20">
                              Certificate
                            </span>
                          )}
                        {course.affiliate && (
                          <span className="px-2 py-1 text-xs font-medium rounded bg-muted text-muted-foreground">
                            Affiliate
                          </span>
                        )}
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        className="gap-2"
                        onClick={() => window.open(course.url, "_blank")}
                      >
                        Visit course
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
