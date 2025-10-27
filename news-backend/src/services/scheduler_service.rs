use anyhow::Result;
use sqlx::PgPool;
use tracing::{error, info};
use crate::services::collector_service::CollectorService;

pub struct SchedulerService {
    db: PgPool,
    collector: CollectorService,
}

impl SchedulerService {
    pub fn new(db: PgPool, collector: CollectorService) -> Self {
        Self { db, collector }
    }

    /// Inicia o scheduler
    pub async fn start(&self) -> Result<()> {
        info!("Starting scheduler service");

        // TODO: Implementar com tokio-cron-scheduler
        // 1. Buscar todos os portais ativos
        // 2. Para cada portal, criar job agendado
        // 3. Executar coleta nos intervalos configurados

        // Placeholder: Executar coleta manual para teste
        self.run_scheduled_tasks().await?;

        Ok(())
    }

    /// Executa tarefas agendadas
    async fn run_scheduled_tasks(&self) -> Result<()> {
        // TODO: Buscar portais ativos do banco
        // Placeholder - struct não implementada ainda
        struct PortalPlaceholder {
            id: i32,
            name: String,
            frequency_minutes: i32,
        }

        let portals: Vec<PortalPlaceholder> = vec![];

        for portal in portals {
            info!(
                portal_id = portal.id,
                portal_name = %portal.name,
                frequency_minutes = portal.frequency_minutes,
                "Scheduling collection task"
            );

            // TODO: Agendar com tokio-cron-scheduler
            // Por enquanto, apenas executar uma vez
            match self.collector.collect_for_portal(portal.id).await {
                Ok(result) => {
                    info!(
                        portal_id = portal.id,
                        documents_collected = result.documents_collected,
                        duration_ms = result.duration_ms,
                        "Collection completed"
                    );
                }
                Err(e) => {
                    error!(
                        portal_id = portal.id,
                        error = %e,
                        "Collection failed"
                    );
                }
            }
        }

        Ok(())
    }
}


