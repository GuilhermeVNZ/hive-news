use crate::filter::parser::ParsedPdf;

#[derive(Debug, Clone, PartialEq)]
pub enum SourceType {
    Scientific,
    NonScientific,
}

const SCIENTIFIC_SOURCES: &[&str] = &[
    "arxiv",
    "nature",
    "science",
    "ieee",
    "springer",
    "elsevier",
    "acm",
    "pubmed",
    "cell",
    "plos",
    "frontiers",
    "mdpi",
    "wiley",
    "taylor-francis",
];

const SCIENTIFIC_DOMAINS: &[&str] = &[
    "arxiv.org",
    "nature.com",
    "science.org",
    "sciencedirect.com",
    "ieee.org",
    "springer.com",
    "acm.org",
    "nih.gov",
    "cell.com",
];

pub fn detect_source_type(parsed: &ParsedPdf) -> SourceType {
    let source_lower = parsed.source_name.to_lowercase();

    // Check fonte da coleta
    for scientific in SCIENTIFIC_SOURCES {
        if source_lower.contains(scientific) {
            return SourceType::Scientific;
        }
    }

    // Check domínio do URL
    for domain in SCIENTIFIC_DOMAINS {
        if parsed.source_url.contains(domain) {
            return SourceType::Scientific;
        }
    }

    // Check se tem DOI (forte indicador de paper científico)
    if !parsed.dois.is_empty() {
        return SourceType::Scientific;
    }

    // Check se tem estrutura de paper (Abstract, References)
    let text_lower = parsed.text.to_lowercase();
    let has_abstract = text_lower.contains("abstract");
    let has_references = text_lower.contains("references") || text_lower.contains("bibliography");

    if has_abstract && has_references {
        return SourceType::Scientific;
    }

    // Default: não-científico (blog, news, etc)
    SourceType::NonScientific
}
