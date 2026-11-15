interface FlagCheck {
  flag: string;
  present: boolean;
}

interface FocusCompletion {
  id: string;
  tag?: string | null;
  bypass?: boolean;
}

interface FocusRoute {
  condition: ScriptCondition<FlagCheck>;
  flags?: FlagCheck[];
  trees: string[];
  completions: FocusCompletion[];
  nested?: FocusRoute[][];
  calls?: [string, string][];
  foreign_events?: { recipient: string; target: string; delay: string }[];
}

interface BackgroundFocus {
  id: string;
  country_tags?: string[];
  initial_tree?: boolean;
  completed_at_start?: boolean;
  never_manually_available?: boolean;
  branch_conditioned?: boolean;
  branch_flags?: FlagCheck[] | null;
  branch_condition?: ScriptCondition<FlagCheck>;
  bypass_condition?: ScriptCondition<FlagCheck>;
  background_availability?: FlagCheck[] | null;
  availability_condition?: ScriptCondition<FlagCheck>;
  flag_changes?: FlagCheck[];
  selection_flags?: FlagCheck[];
  prerequisite_groups?: string[][];
  mutually_exclusive_ids?: string[];
  focus_tree_id?: string;
  focus_tree_changes?: string[];
  selection_routes?: FocusRoute[][];
  selection_completions?: FocusCompletion[];
  conditional_routes?: FocusRoute[][];
  focus_completions?: FocusCompletion[];
  ai_weight?: number;
  duration_days?: number;
  x?: number;
  y?: number;
}

// Produce one reproducible route for each country. Unmodeled gates stop
// individual focuses from entering the projection.
export function projectForeignFocuses<T extends BackgroundFocus>(
  chapters: T[],
  initialFlags: Record<string, string[]>,
  rules: Record<string, string>,
): { owner: string; focus: T; start: number | null; day: number;
  selection: FocusRoute[]; rewards: FocusRoute[]; foreignCompletions: (FocusCompletion & { tag: string; day: number })[];
  bypassed: boolean }[] {
  const countries = new Map<string, T[]>();
  for (const chapter of chapters) {
    const prefix = /^([A-Z0-9]{3})_/.exec(chapter.id)?.[1];
    const owners = chapter.country_tags?.length ? chapter.country_tags : prefix ? [prefix] : [];
    for (const owner of owners) {
      const country = countries.get(owner) ?? [];
      country.push(chapter);
      countries.set(owner, country);
    }
  }
  const result: ReturnType<typeof projectForeignFocuses<T>> = [];
  for (const [owner, chapters] of countries) {
    const initial = chapters.filter((focus) => focus.initial_tree);
    if (!initial.length) continue;
    const flags = new Set(initialFlags[owner] ?? []);
    const completed = new Set(chapters.filter((focus) => focus.completed_at_start).map((focus) => focus.id));
    const excluded = new Set<string>();
    const completedAtStart = initial.filter((focus) => completed.has(focus.id));
    for (const focus of completedAtStart) {
      focus.mutually_exclusive_ids?.forEach((id) => excluded.add(id));
    }
    let activeTrees = new Set(initial.map((focus) => focus.focus_tree_id));
    let day = 0;
    const check = ({ flag, present }: FlagCheck) => {
      if (flag.startsWith('global:')) return null;
      if (flag.startsWith('tag:')) return (owner === flag.slice(4)) === present;
      if (flag === 'is_ai') return present;
      if (flag.startsWith('focus:')) {
        const id = flag.slice(6);
        const separator = id.indexOf(':');
        if (separator !== -1 && id.slice(0, separator) !== owner) return null;
        return completed.has(id.slice(separator + 1)) === present;
      }
      if (flag.startsWith('rule:')) {
        const [rule, value] = flag.slice(5).split('=');
        return rules[rule] === undefined ? null : (rules[rule] === value) === present;
      }
      return flags.has(flag) === present;
    };
    const order = [...chapters].sort((a, b) =>
      (b.ai_weight ?? 1) - (a.ai_weight ?? 1) ||
      (a.y ?? 0) - (b.y ?? 0) ||
      (a.x ?? 0) - (b.x ?? 0) ||
      a.id.localeCompare(b.id),
    );
    const applyFlags = (changes: FlagCheck[] = []) => {
      for (const { flag, present } of changes) {
        if (flag.startsWith('global:')) continue;
        if (present) flags.add(flag);
        else flags.delete(flag);
      }
    };
    const changeTrees = (trees: string[] = []) => {
      for (const tree of trees) {
        if (chapters.some((focus) => focus.focus_tree_id === tree)) activeTrees = new Set([tree]);
      }
    };
    const run = (focus: T, start: number | null, bypassed = false) => {
      const selection: FocusRoute[] = [], rewards: FocusRoute[] = [];
      const foreignCompletions: (FocusCompletion & { tag: string; day: number })[] = [];
      const complete = (targets: FocusCompletion[] = []) => {
        for (const { tag, id, bypass } of targets) {
          if (tag && tag !== owner) {
            foreignCompletions.push({ tag, id, day, bypass });
            continue;
          }
          if (completed.has(id)) continue;
          const target = chapters.find((candidate) => candidate.id === id && activeTrees.has(candidate.focus_tree_id));
          if (target) run(target, null, bypass);
        }
      };
      const applyRoutes = (chains: FocusRoute[][] = [], executed: FocusRoute[]) => {
        for (const chain of chains) {
          for (const route of chain) {
            const matches = evaluateCondition(route.condition, check);
            if (matches === false) continue;
            if (matches === true) {
              executed.push(route);
              applyFlags(route.flags);
              changeTrees(route.trees);
              complete(route.completions);
              applyRoutes(route.nested, executed);
            }
            break;
          }
        }
      };
      if (start !== null) {
        applyFlags(focus.selection_flags);
        complete(focus.selection_completions);
        applyRoutes(focus.selection_routes, selection);
        day += Math.max(1, focus.duration_days ?? 70);
      }
      completed.add(focus.id);
      focus.mutually_exclusive_ids?.forEach((id) => excluded.add(id));
      if (!bypassed) {
        applyFlags(focus.flag_changes);
        changeTrees(focus.focus_tree_changes);
        complete(focus.focus_completions);
        applyRoutes(focus.conditional_routes, rewards);
      }
      result.push({ owner, focus, start, day, selection, rewards, foreignCompletions, bypassed });
    };
    for (;;) {
      const reachable = (focus: T) =>
        activeTrees.has(focus.focus_tree_id) && !completed.has(focus.id) && !excluded.has(focus.id) &&
        !focus.mutually_exclusive_ids?.some((id) => completed.has(id)) &&
        (!focus.branch_conditioned || (focus.branch_condition
          ? evaluateCondition(focus.branch_condition, check) === true
          : !!focus.branch_flags?.every(check))) &&
        (focus.prerequisite_groups ?? []).every((group) => group.some((id) => completed.has(id)));
      const bypass = order.find((focus) => focus.bypass_condition && reachable(focus) &&
        evaluateCondition(focus.bypass_condition, check) === true);
      if (bypass) {
        run(bypass, null, true);
        continue;
      }
      const next = order.find((focus) => reachable(focus) && !focus.never_manually_available && (focus.ai_weight ?? 1) > 0 &&
        (focus.availability_condition ? evaluateCondition(focus.availability_condition, check) === true
          : !!focus.background_availability?.every(check)),
      );
      if (!next) break;
      run(next, day);
    }
  }
  return result;
}
import { evaluateCondition, ScriptCondition } from './script-condition';

