use anyhow::{Context, Result};
use reqwest::Client;
use crate::models::course::Course;
use serde::Deserialize;

pub struct CourseCollector {
    client: Client,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct KhanAcademyCourse {
    id: String,
    title: String,
    description: Option<String>,
    url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct KhanAcademyResponse {
    courses: Vec<KhanAcademyCourse>,
}

impl CourseCollector {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client })
    }

    /// Busca cursos de IA/ML da Khan Academy
    #[allow(dead_code)]
    pub async fn fetch_khan_academy_courses(&self) -> Result<Vec<Course>> {
        // Khan Academy tem API pública mas limitada
        // Vamos buscar informações estruturadas do site
        println!("🔍 Fetching courses from Khan Academy...");
        
        // Nota: Khan Academy não tem API pública oficial para lista de cursos
        // Você pode precisar usar web scraping ou entrar em contato com eles
        
        Ok(vec![])
    }

    /// Busca cursos do MIT OpenCourseWare (dados estruturados)
    pub async fn fetch_mit_ocw_courses(&self) -> Result<Vec<Course>> {
        println!("🔍 Fetching courses from MIT OpenCourseWare...");
        
        // MIT OCW tem dados em formato JSON disponíveis
        // Exemplo: https://ocw.mit.edu/courses/artificial-intelligence/data.json
        
        let course_ids = vec![
            "6-034-artificial-intelligence-fall-2010",
            "6-0002-introduction-to-computational-thinking-and-data-science-fall-2016",
            "6-867-machine-learning-fall-2006",
        ];

        let mut courses = Vec::new();

        for course_id in course_ids {
            let url = format!("https://ocw.mit.edu/courses/{}/data.json", course_id);
            
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(data) => {
                                if let Some(title) = data["title"].as_str() {
                                    let description = data["description"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string();
                                    
                                    courses.push(Course {
                                        id: course_id.to_string(),
                                        title: title.to_string(),
                                        platform: "MIT OpenCourseWare".to_string(),
                                        instructor: data["instructors"]
                                            .as_array()
                                            .and_then(|arr| arr.first())
                                            .and_then(|instr| instr["first_name"].as_str())
                                            .unwrap_or("MIT Faculty")
                                            .to_string(),
                                        institution: Some("MIT".to_string()),
                                        category: extract_category_from_title(title),
                                        duration: format!("{} weeks", 
                                            data["term"].as_str().unwrap_or("")),
                                        level: "Intermediário".to_string(),
                                        price: "Grátis".to_string(),
                                        rating: None,
                                        students: None,
                                        description: if description.is_empty() {
                                            format!("Curso do MIT sobre {}", title)
                                        } else {
                                            description.chars().take(200).collect()
                                        },
                                        url: format!("https://ocw.mit.edu/courses/{}", course_id),
                                        language: Some("English".to_string()),
                                        image_url: None,
                                        affiliate: false,
                                        certificate_available: false,
                                        free: true,
                                    });
                                }
                            }
                            Err(e) => {
                                eprintln!("   ⚠️  Failed to parse MIT OCW JSON for {}: {}", course_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️  Failed to fetch MIT OCW course {}: {}", course_id, e);
                }
            }
            
            // Delay entre requisições
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        println!("   ✅ Found {} courses from MIT OCW", courses.len());
        Ok(courses)
    }

    /// Busca cursos públicos de outras fontes (placeholder)
    pub async fn fetch_public_courses(&self) -> Result<Vec<Course>> {
        println!("🔍 Fetching courses from public APIs...");
        
        let mut all_courses = Vec::new();

        // MIT OCW
        match self.fetch_mit_ocw_courses().await {
            Ok(mut courses) => {
                all_courses.append(&mut courses);
            }
            Err(e) => {
                eprintln!("   ⚠️  Failed to fetch MIT OCW courses: {}", e);
            }
        }

        // Adicionar mais fontes aqui conforme você obtém acesso às APIs
        
        println!("   ✅ Total: {} courses from public sources", all_courses.len());
        Ok(all_courses)
    }
}

fn extract_category_from_title(title: &str) -> String {
    let title_lower = title.to_lowercase();
    
    if title_lower.contains("artificial intelligence") || title_lower.contains("ai") {
        "Introdução à IA".to_string()
    } else if title_lower.contains("machine learning") || title_lower.contains("ml") {
        "Machine Learning".to_string()
    } else if title_lower.contains("neural network") || title_lower.contains("deep learning") {
        "Deep Learning".to_string()
    } else if title_lower.contains("computer vision") || title_lower.contains("vision") {
        "Computer Vision".to_string()
    } else if title_lower.contains("natural language") || title_lower.contains("nlp") {
        "NLP".to_string()
    } else if title_lower.contains("data science") || title_lower.contains("data") {
        "Data Science".to_string()
    } else {
        "Machine Learning".to_string()
    }
}

