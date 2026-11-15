export type ScriptCondition<T> =
  | { op: 'check'; args: T }
  | { op: 'all' | 'any'; args: ScriptCondition<T>[] }
  | { op: 'not'; args: ScriptCondition<T> }
  | { op: 'unknown' };

export function evaluateCondition<T>(
  condition: ScriptCondition<T>,
  check: (value: T) => boolean | null,
): boolean | null {
  switch (condition.op) {
    case 'check': return check(condition.args);
    case 'unknown': return null;
    case 'not': {
      const result = evaluateCondition(condition.args, check);
      return result === null ? null : !result;
    }
    case 'all':
    case 'any': {
      const any = condition.op === 'any';
      let unknown = false;
      for (const term of condition.args) {
        const result = evaluateCondition(term, check);
        if (result === any) return any;
        unknown ||= result === null;
      }
      return unknown ? null : !any;
    }
  }
}
