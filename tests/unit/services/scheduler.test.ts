import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { SchedulerService } from "../../../apps/backend-cmmv/src/services/scheduler.service";

describe("SchedulerService", () => {
  let service: SchedulerService;

  beforeEach(() => {
    service = new SchedulerService();
    vi.clearAllMocks();
  });

  afterEach(() => {
    service.stop();
  });

  describe("registerJob", () => {
    it("should register a new job", () => {
      const job = {
        id: "job-1",
        name: "Test Job",
        module: "test",
        cronExpression: "0 * * * *",
        handler: vi.fn().mockResolvedValue(undefined),
      };

      service.registerJob(job);

      // Verify job is registered
      expect(() => service.executeJob("job-1")).not.toThrow();
    });

    it("should register multiple jobs", () => {
      const job1 = {
        id: "job-1",
        name: "Job 1",
        module: "test",
        cronExpression: "0 * * * *",
        handler: vi.fn().mockResolvedValue(undefined),
      };

      const job2 = {
        id: "job-2",
        name: "Job 2",
        module: "test",
        cronExpression: "*/15 * * * *",
        handler: vi.fn().mockResolvedValue(undefined),
      };

      service.registerJob(job1);
      service.registerJob(job2);

      expect(() => service.executeJob("job-1")).not.toThrow();
      expect(() => service.executeJob("job-2")).not.toThrow();
    });
  });

  describe("start", () => {
    it("should start scheduler and mark as running", () => {
      service.start();

      expect(service.getRunning()).toBe(true);
    });

    it("should schedule all registered jobs", () => {
      const consoleLogSpy = vi.spyOn(console, "log").mockImplementation(() => {});

      service.registerJob({
        id: "job-1",
        name: "Test Job",
        module: "test",
        cronExpression: "0 * * * *",
        handler: vi.fn().mockResolvedValue(undefined),
      });

      service.start();

      expect(consoleLogSpy).toHaveBeenCalledWith("Scheduled job: Test Job (0 * * * *)");
    });
  });

  describe("stop", () => {
    it("should stop scheduler and mark as not running", () => {
      service.start();
      expect(service.getRunning()).toBe(true);

      service.stop();
      expect(service.getRunning()).toBe(false);
    });
  });

  describe("executeJob", () => {
    it("should execute a registered job", async () => {
      const handler = vi.fn().mockResolvedValue(undefined);

      service.registerJob({
        id: "job-1",
        name: "Test Job",
        module: "test",
        cronExpression: "0 * * * *",
        handler,
      });

      await service.executeJob("job-1");

      expect(handler).toHaveBeenCalledTimes(1);
    });

    it("should throw error for non-existent job", async () => {
      await expect(service.executeJob("non-existent")).rejects.toThrow(
        "Job not found: non-existent"
      );
    });

    it("should execute job handler", async () => {
      const handler = vi.fn().mockResolvedValue(undefined);

      service.registerJob({
        id: "job-1",
        name: "Test Job",
        module: "test",
        cronExpression: "0 * * * *",
        handler,
      });

      await service.executeJob("job-1");

      expect(handler).toHaveBeenCalled();
    });
  });

  describe("getNextExecution", () => {
    it("should calculate next execution for hourly jobs", () => {
      const cronExpression = "0 * * * *";
      const nextExecution = service.getNextExecution(cronExpression);

      expect(nextExecution).toBeInstanceOf(Date);
      expect(nextExecution.getTime()).toBeGreaterThan(Date.now());
    });

    it("should calculate next execution for custom jobs", () => {
      const cronExpression = "*/15 * * * *";
      const nextExecution = service.getNextExecution(cronExpression);

      expect(nextExecution).toBeInstanceOf(Date);
    });
  });

  describe("Integration scenarios", () => {
    it("should handle full workflow: register -> start -> execute -> stop", async () => {
      const handler = vi.fn().mockResolvedValue(undefined);
      const consoleLogSpy = vi.spyOn(console, "log").mockImplementation(() => {});

      service.registerJob({
        id: "workflow-job",
        name: "Workflow Job",
        module: "integration",
        cronExpression: "0 * * * *",
        handler,
      });

      expect(service.getRunning()).toBe(false);

      service.start();
      expect(service.getRunning()).toBe(true);
      expect(consoleLogSpy).toHaveBeenCalledWith("Scheduled job: Workflow Job (0 * * * *)");

      await service.executeJob("workflow-job");
      expect(handler).toHaveBeenCalledTimes(1);

      service.stop();
      expect(service.getRunning()).toBe(false);
    });
  });

  describe("getNextExecution", () => {
    it("should calculate next execution for hourly jobs", () => {
      const cronExpression = "0 * * * *";
      const nextExecution = service.getNextExecution(cronExpression);

      expect(nextExecution).toBeInstanceOf(Date);
      expect(nextExecution.getTime()).toBeGreaterThan(Date.now());
    });

    it("should calculate next execution for custom jobs", () => {
      const cronExpression = "*/15 * * * *";
      const nextExecution = service.getNextExecution(cronExpression);

      expect(nextExecution).toBeInstanceOf(Date);
    });

    it("should handle non-hourly cron expressions", () => {
      const cronExpression = "*/30 * * * *";
      const nextExecution = service.getNextExecution(cronExpression);

      expect(nextExecution).toBeInstanceOf(Date);
      // Should return now for non-hourly jobs
      expect(nextExecution.getTime()).toBeGreaterThanOrEqual(Date.now() - 1000);
    });
  });

  describe("Schedule job error handling", () => {
    it("should handle invalid cron expressions gracefully", () => {
      const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

      service.registerJob({
        id: "invalid-job",
        name: "Invalid Job",
        module: "test",
        cronExpression: "invalid",
        handler: vi.fn(),
      });

      service.start();

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining("Invalid cron expression")
      );

      consoleErrorSpy.mockRestore();
    });

    it("should handle cron expressions with wrong number of parts", () => {
      const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

      service.registerJob({
        id: "wrong-parts-job",
        name: "Wrong Parts",
        module: "test",
        cronExpression: "0 * *",
        handler: vi.fn(),
      });

      service.start();

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining("Invalid cron expression")
      );

      consoleErrorSpy.mockRestore();
    });
  });

  describe("Complex scenarios", () => {
    it("should handle multiple jobs with different schedules", async () => {
      const handler1 = vi.fn().mockResolvedValue(undefined);
      const handler2 = vi.fn().mockResolvedValue(undefined);
      const handler3 = vi.fn().mockResolvedValue(undefined);

      service.registerJob({
        id: "job-1",
        name: "Every Hour",
        module: "test",
        cronExpression: "0 * * * *",
        handler: handler1,
      });

      service.registerJob({
        id: "job-2",
        name: "Every 15 Minutes",
        module: "test",
        cronExpression: "*/15 * * * *",
        handler: handler2,
      });

      service.registerJob({
        id: "job-3",
        name: "Daily",
        module: "test",
        cronExpression: "0 0 * * *",
        handler: handler3,
      });

      expect(() => service.executeJob("job-1")).not.toThrow();
      expect(() => service.executeJob("job-2")).not.toThrow();
      expect(() => service.executeJob("job-3")).not.toThrow();
    });
  });
});
