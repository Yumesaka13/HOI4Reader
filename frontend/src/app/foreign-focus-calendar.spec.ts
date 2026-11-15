import { projectForeignFocuses } from './foreign-focus-calendar';

describe('foreign focus calendar', () => {
  it('chooses a weighted route and leaves unsupported or excluded focuses unscheduled', () => {
    const root = {
      id: 'SOV_root',
      focus_tree_id: 'sov_start',
      initial_tree: true,
      background_availability: [],
      duration_days: 7,
      flag_changes: [{ flag: 'route_unlocked', present: true }],
    };
    const branch = {
      ...root,
      prerequisite_groups: [[root.id]],
      background_availability: [{ flag: 'route_unlocked', present: true }],
    };
    const focuses = [
      { ...root, id: 'SOV_unknown', background_availability: null, ai_weight: 100 },
      { ...branch, id: 'SOV_right', ai_weight: 1 },
      { ...branch, id: 'SOV_left', ai_weight: 2, mutually_exclusive_ids: ['SOV_right'] },
      root,
    ];

    expect(projectForeignFocuses(focuses, {}, {}).map(({ owner, focus, start, day }) =>
      [owner, focus.id, start, day],
    )).toEqual([
      ['SOV', 'SOV_root', 0, 7],
      ['SOV', 'SOV_left', 7, 14],
    ]);
  });

  it('runs a scripted completion and its follow-up without adding another focus duration', () => {
    const base = { focus_tree_id: 'test', initial_tree: true, background_availability: [], duration_days: 7 };
    const focuses = [
      { ...base, id: 'GER_root', ai_weight: 10, focus_completions: [{ id: 'GER_automatic' }, { id: 'GER_skipped', bypass: true }] },
      { ...base, id: 'GER_automatic', never_manually_available: true, flag_changes: [{ flag: 'ready', present: true }] },
      { ...base, id: 'GER_skipped', never_manually_available: true, flag_changes: [{ flag: 'unwanted', present: true }] },
      { ...base, id: 'GER_next', prerequisite_groups: [['GER_automatic'], ['GER_skipped']],
        background_availability: [{ flag: 'ready', present: true }, { flag: 'unwanted', present: false }] },
    ];
    const route = projectForeignFocuses(focuses, {}, {});
    expect(route.map(({ focus, start, day, bypassed }) => [focus.id, start, day, bypassed])).toEqual([
      ['GER_automatic', null, 7, false], ['GER_skipped', null, 7, true],
      ['GER_root', 0, 7, false], ['GER_next', 7, 14, false],
    ]);
  });
});
