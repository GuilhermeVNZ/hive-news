use axum::{extract::Query, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::collectors::course_collector::CourseCollector;
use crate::models::course::Course;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CourseQuery {
    pub category: Option<String>,
    pub platform: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CoursesResponse {
    pub courses: Vec<Course>,
    pub total: usize,
    pub platforms: Vec<String>,
    pub categories: Vec<String>,
}

pub fn router() -> Router {
    Router::new().route("/", get(get_courses))
}

async fn get_courses(Query(params): Query<HashMap<String, String>>) -> Json<CoursesResponse> {
    println!("📚 Fetching courses from public APIs...");

    let collector = match CourseCollector::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to create course collector: {}", e);
            return Json(CoursesResponse {
                courses: vec![],
                total: 0,
                platforms: vec![],
                categories: vec![],
            });
        }
    };

    // Buscar cursos de APIs públicas
    let mut all_courses = match collector.fetch_public_courses().await {
        Ok(courses) => courses,
        Err(e) => {
            eprintln!("⚠️  Failed to fetch public courses: {}", e);
            vec![]
        }
    };

    // Aplicar filtros se fornecidos
    if let Some(category) = params.get("category") {
        if category != "all" {
            all_courses.retain(|c| c.category.to_lowercase() == category.to_lowercase());
        }
    }

    if let Some(platform) = params.get("platform") {
        all_courses.retain(|c| c.platform.to_lowercase() == platform.to_lowercase());
    }

    if let Some(language) = params.get("language") {
        all_courses.retain(|c| {
            c.language
                .as_ref()
                .map(|l| l.to_lowercase() == language.to_lowercase())
                .unwrap_or(true)
        });
    }

    // Extrair plataformas e categorias únicas
    let platforms: Vec<String> = all_courses
        .iter()
        .map(|c| c.platform.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    let categories: Vec<String> = all_courses
        .iter()
        .map(|c| c.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    println!("✅ Returning {} courses", all_courses.len());

    Json(CoursesResponse {
        total: all_courses.len(),
        courses: all_courses,
        platforms,
        categories,
    })
}

