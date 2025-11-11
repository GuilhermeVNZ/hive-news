use reqwest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔬 Test Collector - Single Paper");
    println!("=====================================\n");

    println!("📥 Fetching single paper from arXiv...");
    println!("   Category: cs.AI\n");

    // Cliente HTTP
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    // URL da API do arXiv (mesma lógica do collector)
    let url = format!(
        "https://export.arxiv.org/api/query?search_query=cat:cs.AI&start=0&max_results=1&sortBy=submittedDate&sortOrder=descending"
    );

    println!("📡 URL: {}", url);

    // Fazer requisição
    let response = client.get(&url).send().await?;
    let xml = response.text().await?;

    println!("📄 Response length: {} bytes\n", xml.len());

    // Parse do XML (mesma lógica do main.rs)
    let mut current_id = None;
    let mut current_title = None;

    for line in xml.lines() {
        if line.contains("<id>") {
            if let Some(start) = line.find("<id>") {
                if let Some(end) = line.find("</id>") {
                    let id = &line[start + 4..end];
                    if id.contains("arxiv.org/abs/") {
                        let mut paper_id = id
                            .replace("http://arxiv.org/abs/", "")
                            .replace("https://arxiv.org/abs/", "");
                        // Remove version suffix (v1, v2, etc.) to get published version
                        if let Some(pos) = paper_id.rfind('v') {
                            paper_id = paper_id[..pos].to_string();
                        }
                        current_id = Some(paper_id);
                    }
                }
            }
        }
        if line.contains("<title>") && current_id.is_some() {
            if let Some(start) = line.find("<title>") {
                if let Some(end) = line.find("</title>") {
                    let title = &line[start + 7..end];
                    current_title = Some(title.to_string());
                }
            }
        }
    }

    // Exibir resultados
    if let Some(paper_id) = current_id {
        let title = current_title.unwrap_or_else(|| "Untitled".to_string());

        // Gerar URLs (mesma lógica do collector)
        let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", paper_id);
        let abstract_url = format!("https://arxiv.org/abs/{}", paper_id);

        println!("{}", "=".repeat(80));
        println!("📄 PAPER ENCONTRADO:");
        println!("{}", "=".repeat(80));
        println!("   Titulo: {}", title);
        println!("   ID: {}", paper_id);
        println!();
        println!("LINKS:");
        println!("   PDF: {}", pdf_url);
        println!("   Abstract: {}", abstract_url);
        println!();
        println!("{}", "=".repeat(80));
        println!("COPIE E COLE ESTE LINK NO SEU NAVEGADOR:");
        println!("   {}", pdf_url);
        println!("{}", "=".repeat(80));
    } else {
        println!("Erro: Nao foi possivel encontrar o ID do paper no XML");
    }

    Ok(())
}
