use crate::filter::parser::ParsedPdf;

pub struct FilterResult {
    pub doc: ParsedPdf,
    pub doi_ratio: f32,
    pub author_ratio: f32,
    pub has_exp: bool,
    pub fake_penalty: f32,
}

pub fn calculate_score(result: &FilterResult) -> f32 {
    let doi_score = result.doi_ratio * 0.4;
    let exp_score = if result.has_exp { 0.3 } else { 0.0 };
    let author_score = result.author_ratio * 0.2;
    let fake_score = (1.0 - result.fake_penalty) * 0.1;
    
    doi_score + exp_score + author_score + fake_score
}


