/**
 * Scheduler Service
 * Manages cron jobs and task scheduling
 */

export interface JobDefinition {
  id: string;
  name: string;
  module: string;
  cronExpression: string;
  handler: () => Promise<void>;
}

export class SchedulerService {
  private jobs: Map<string, JobDefinition> = new Map();
  private _running = false;

  /**
   * Register a scheduled job
   */
  registerJob(definition: JobDefinition): void {
    this.jobs.set(definition.id, definition);
  }

  /**
   * Start all scheduled jobs
   */
  start(): void {
    this._running = true;
    this.scheduleJobs();
  }

  /**
   * Stop all scheduled jobs
   */
  stop(): void {
    this._running = false;
  }

  /**
   * Execute a job now (without waiting for schedule)
   */
  async executeJob(jobId: string): Promise<void> {
    const job = this.jobs.get(jobId);
    if (!job) throw new Error(`Job not found: ${jobId}`);

    await job.handler();
  }

  private scheduleJobs(): void {
    for (const [, job] of this.jobs.entries()) {
      this.scheduleJob(job);
    }
  }

  private scheduleJob(job: JobDefinition): void {
    // Parse cron expression
    const cronParts = job.cronExpression.split(/\s+/);

    if (cronParts.length !== 5) {
      console.error(`Invalid cron expression: ${job.cronExpression}`);
      return;
    }

    // For now, just log the scheduling
    console.log(`Scheduled job: ${job.name} (${job.cronExpression})`);

    // In production, use node-cron or similar library
    // cron.schedule(job.cronExpression, job.handler);
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  getRunning(): boolean {
    return this._running;
  }

  /**
   * Calculate next execution time from cron expression
   */
  getNextExecution(cronExpression: string): Date {
    // Parse cron and calculate next run time
    const now = new Date();

    // Simplified: add 1 hour for hourly jobs
    if (cronExpression.startsWith("0 *")) {
      return new Date(now.getTime() + 3600000);
    }

    return now;
  }
}
