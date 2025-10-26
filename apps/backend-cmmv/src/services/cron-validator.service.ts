/**
 * Cron Validator Service
 * Validates and parses cron expressions for publishing cadence
 */

export class CronValidatorService {
  /**
   * Validate cron syntax
   * Format: minute hour day month dayOfWeek
   */
  validateCronSyntax(expression: string): boolean {
    const parts = expression.trim().split(/\s+/);

    if (parts.length !== 5) {
      return false;
    }

    const [minute, hour, day, month, dayOfWeek] = parts;

    return (
      this.validateMinute(minute) &&
      this.validateHour(hour) &&
      this.validateDay(day) &&
      this.validateMonth(month) &&
      this.validateDayOfWeek(dayOfWeek)
    );
  }

  /**
   * Parse and validate cron expression
   */
  parseCron(expression: string): {
    minute: string;
    hour: string;
    day: string;
    month: string;
    dayOfWeek: string;
  } | null {
    if (!this.validateCronSyntax(expression)) {
      return null;
    }

    const parts = expression.trim().split(/\s+/);
    return {
      minute: parts[0],
      hour: parts[1],
      day: parts[2],
      month: parts[3],
      dayOfWeek: parts[4],
    };
  }

  /**
   * Check if cron is valid for publication cadence
   * Examples: "0 * * * *" (every hour), "*\/15 * * * *" (every 15 min)
   */
  isValidCadence(cadence: string): boolean {
    return this.validateCronSyntax(cadence);
  }

  private validateMinute(minute: string): boolean {
    const basePattern = /^(\*|[0-5]?\d)$/;
    const intervalPattern = /^\*\/\d+$/;
    const rangePattern = /^[0-5]?\d-\d$/;
    return basePattern.test(minute) || intervalPattern.test(minute) || rangePattern.test(minute);
  }

  private validateHour(hour: string): boolean {
    const basePattern = /^(\*|[01]?\d|2[0-3])$/;
    const intervalPattern = /^\*\/\d+$/;
    const rangePattern = /^[01]?\d|2[0-3]-\d$/;
    return basePattern.test(hour) || intervalPattern.test(hour) || rangePattern.test(hour);
  }

  private validateDay(day: string): boolean {
    return /^(\*|[12]?\d|3[01])$/.test(day);
  }

  private validateMonth(month: string): boolean {
    return /^(\*|[01]?\d)$/.test(month);
  }

  private validateDayOfWeek(dayOfWeek: string): boolean {
    return /^(\*|[0-6])$/.test(dayOfWeek);
  }
}
