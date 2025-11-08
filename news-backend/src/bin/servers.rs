//! Script orquestrador para iniciar todos os servidores do sistema
//! 
//! Este script:
//! 1. Finaliza todos os processos em execução
//! 2. Aguarda 10 segundos
//! 3. Inicia News Dashboard (localhost:1420)
//! 4. Inicia AIResearch (localhost:3003)
//! 5. Inicia ScienceAI (localhost:8080)
//! 6. Inicia Backend (localhost:3005)
//! 7. Verifica system_config.json para outros serviços

use std::process::{Command, Stdio};
use std::path::Path;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🚀 Iniciando orquestrador de servidores...");
    println!("=============================================");
    println!();

    // 1. Finalizar todos os processos
    println!("1️⃣  Finalizando todos os processos em execução...");
    kill_all_processes();
    println!();

    // 2. Aguardar 10 segundos
    println!("2️⃣  Aguardando 10 segundos...");
    thread::sleep(Duration::from_secs(10));
    println!();

    // 3. Iniciar News Dashboard (localhost:1420)
    println!("3️⃣  Iniciando News Dashboard (localhost:1420)...");
    start_news_dashboard();
    thread::sleep(Duration::from_secs(2));
    println!();

    // 4. Iniciar AIResearch (localhost:3003)
    println!("4️⃣  Iniciando AIResearch (localhost:3003)...");
    start_airesearch();
    thread::sleep(Duration::from_secs(2));
    println!();

    // 5. Iniciar ScienceAI (localhost:8080)
    println!("5️⃣  Iniciando ScienceAI (localhost:8080)...");
    start_scienceai();
    thread::sleep(Duration::from_secs(2));
    println!();

    // 6. Iniciar Backend (localhost:3005)
    println!("6️⃣  Iniciando Backend (localhost:3005)...");
    start_backend();
    thread::sleep(Duration::from_secs(2));
    println!();

    // 7. Verificar system_config.json para outros serviços
    println!("7️⃣  Verificando system_config.json para outros serviços...");
    check_additional_services();
    println!();

    println!("✅ Orquestração concluída!");
    println!();
    println!("📊 Servidores iniciados:");
    println!("   - News Dashboard: http://localhost:1420");
    println!("   - AIResearch: http://localhost:3003");
    println!("   - ScienceAI: http://localhost:8080");
    println!("   - Backend: http://localhost:3005");
    println!();
}

fn kill_all_processes() {
    // Executar script PowerShell para encerrar processos
    let script_path = Path::new("G:/Hive-Hub/News-main/kill-all-processes.ps1");
    
    if script_path.exists() {
        let output = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(script_path)
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("   ✅ Processos finalizados");
                } else {
                    println!("   ⚠️  Alguns processos podem não ter sido finalizados");
                }
            }
            Err(e) => {
                println!("   ⚠️  Erro ao executar script de finalização: {}", e);
            }
        }
    } else {
        println!("   ⚠️  Script kill-all-processes.ps1 não encontrado");
        println!("   💡 Tentando encerrar processos manualmente...");
        
        // Encerrar processos manualmente via PowerShell
        let commands = vec![
            "Get-Process | Where-Object { $_.Path -like '*news-backend*' -or $_.Path -like '*ScienceAI*' -or $_.Path -like '*frontend-next*' -or ($_.ProcessName -eq 'cargo' -and $_.Path -like '*News-main*') -or ($_.ProcessName -eq 'node' -and ($_.Path -like '*News-main*' -or $_.Path -like '*ScienceAI*' -or $_.Path -like '*frontend-next*')) } | Stop-Process -Force -ErrorAction SilentlyContinue",
        ];
        
        for cmd in commands {
            let _ = Command::new("powershell")
                .arg("-Command")
                .arg(cmd)
                .output();
        }
        
        println!("   ✅ Tentativa de encerrar processos concluída");
    }
}

fn start_news_dashboard() {
    let dashboard_dir = Path::new("G:/Hive-Hub/News-main/news-dashboard");
    
    if !dashboard_dir.exists() {
        println!("   ❌ Diretório do News Dashboard não encontrado: {:?}", dashboard_dir);
        return;
    }
    
    // Iniciar em nova janela PowerShell
    let cmd = format!(
        "cd G:\\Hive-Hub\\News-main\\news-dashboard; Write-Host 'News Dashboard (Port 1420)' -ForegroundColor Cyan; npm run dev"
    );
    
    let output = Command::new("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match output {
        Ok(_) => println!("   ✅ News Dashboard iniciado em nova janela"),
        Err(e) => println!("   ❌ Erro ao iniciar News Dashboard: {}", e),
    }
}

fn start_airesearch() {
    let airesearch_dir = Path::new("G:/Hive-Hub/News-main/apps/frontend-next/AIResearch");
    
    if !airesearch_dir.exists() {
        println!("   ❌ Diretório do AIResearch não encontrado: {:?}", airesearch_dir);
        return;
    }
    
    // Iniciar em nova janela PowerShell
    let cmd = format!(
        "cd G:\\Hive-Hub\\News-main\\apps\\frontend-next\\AIResearch; Write-Host 'AIResearch (Port 3003)' -ForegroundColor Cyan; npm run dev"
    );
    
    let output = Command::new("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match output {
        Ok(_) => println!("   ✅ AIResearch iniciado em nova janela"),
        Err(e) => println!("   ❌ Erro ao iniciar AIResearch: {}", e),
    }
}

fn start_scienceai() {
    let scienceai_dir = Path::new("G:/Hive-Hub/News-main/apps/frontend-next/ScienceAI");
    
    if !scienceai_dir.exists() {
        println!("   ❌ Diretório do ScienceAI não encontrado: {:?}", scienceai_dir);
        return;
    }
    
    // Iniciar em nova janela PowerShell
    let cmd = format!(
        "cd G:\\Hive-Hub\\News-main\\apps\\frontend-next\\ScienceAI; Write-Host 'ScienceAI (Port 8080)' -ForegroundColor Cyan; npm run dev"
    );
    
    let output = Command::new("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match output {
        Ok(_) => println!("   ✅ ScienceAI iniciado em nova janela"),
        Err(e) => println!("   ❌ Erro ao iniciar ScienceAI: {}", e),
    }
}

fn start_backend() {
    // Try to find backend binary in release or debug build
    let backend_paths = vec![
        Path::new("G:/Hive-Hub/News-main/news-backend/target/release/news-backend.exe"),
        Path::new("G:/Hive-Hub/News-main/news-backend/target/debug/news-backend.exe"),
    ];
    
    let backend_path = backend_paths.iter()
        .find(|p| p.exists());
    
    let backend_dir = Path::new("G:/Hive-Hub/News-main/news-backend");
    
    if !backend_dir.exists() {
        println!("   ❌ Diretório do Backend não encontrado: {:?}", backend_dir);
        return;
    }
    
    // Iniciar em nova janela PowerShell
    let cmd = if let Some(path) = backend_path {
        // Use compiled binary directly
        format!(
            "cd G:\\Hive-Hub\\News-main\\news-backend; Write-Host 'News Backend (Port 3005)' -ForegroundColor Cyan; {}",
            path.to_string_lossy()
        )
    } else {
        // Fallback to cargo run if binary not found
        println!("   ⚠️  Backend binary not found, using cargo run --release (will compile)...");
        format!(
            "cd G:\\Hive-Hub\\News-main\\news-backend; Write-Host 'News Backend (Port 3005)' -ForegroundColor Cyan; cargo run --release --bin news-backend"
        )
    };
    
    let output = Command::new("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match output {
        Ok(_) => {
            if backend_path.is_some() {
                println!("   ✅ Backend iniciado em nova janela (usando binário compilado)");
            } else {
                println!("   ✅ Backend iniciando em nova janela (compilando...)");
            }
        },
        Err(e) => println!("   ❌ Erro ao iniciar Backend: {}", e),
    }
}

fn check_additional_services() {
    let config_path = Path::new("G:/Hive-Hub/News-main/news-backend/system_config.json");
    
    if !config_path.exists() {
        println!("   ⚠️  system_config.json não encontrado: {:?}", config_path);
        return;
    }
    
    // Ler e analisar system_config.json
    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            // Tentar parsear JSON básico (sem usar serde aqui para evitar dependências extras)
            // Por enquanto, apenas verificar se há referências a outros serviços
            if content.contains("\"sites\"") {
                println!("   ✅ system_config.json encontrado");
                println!("   📋 Sites configurados detectados no system_config.json");
                println!("   💡 Todos os serviços principais já foram iniciados");
            } else {
                println!("   ⚠️  system_config.json não contém configuração de sites");
            }
        }
        Err(e) => {
            println!("   ⚠️  Erro ao ler system_config.json: {}", e);
        }
    }
}



