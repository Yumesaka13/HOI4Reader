import { TestBed } from '@angular/core/testing';
import type { Channel } from '@tauri-apps/api/core';
import { clearMocks, mockConvertFileSrc, mockIPC } from '@tauri-apps/api/mocks';
import { App } from './app';

describe('App', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [App],
    })
      .compileComponents();
  });

  it('should create the app', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    expect(app).toBeTruthy();
  });

  it('moves effect previews with the pointer and preserves colour and flag tokens', () => {
    const app = TestBed.createComponent(App).componentInstance;
    const option = { id: 'sample.a', text: 'Continue', next_ids: [],
      effect_preview: { lines: [{ text: '@JAP §G+1§!', context: [] }], incomplete: false } };
    app.showEffectTooltip(new MouseEvent('mousemove', { clientX: 20, clientY: 30 }), option);
    expect(app.effectTooltip).toMatchObject({ x: 34, y: 46 });
    app.showEffectTooltip(new MouseEvent('mousemove', { clientX: 35, clientY: 50 }), option);
    expect(app.effectTooltip).toMatchObject({ x: 49, y: 66 });
    const tokens = app.gameText(app.effectTooltip!.text);
    expect(tokens).toContainEqual({ text: 'JAP', flag: true, icon: 'flag:JAP', color: undefined });
    expect(tokens).toContainEqual({ text: '+1', color: '#51cc73' });
  });

  it('shows searchable countries during indexing, opens the selected country once, and reuses the result', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.selectedMod = { name: 'Test mod', path: '/test-mod', workshop_id: '' };
    app.setPage('countries');
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    const open = vi.spyOn(app, 'openRoutePage').mockImplementation(() => {});
    const country = { tag: 'JAP', name: 'Japan', chapter_count: 2, story_count: 3 };
    const index = { start_date: '2020.1.1', countries: [country], chapters: [], timeline: [], diagnostics: [],
      country_names: [country, { tag: 'BHU', name: '不丹' }] };
    let publish!: Channel<{ name: string; tag: string }[]>;
    let finish!: (index: unknown) => void;
    const backend = vi.fn((_, args) => {
      publish = (args as { onCountries: typeof publish }).onCountries;
      return new Promise((resolve) => { finish = resolve; });
    });
    mockIPC(backend);
    try {
      const scan = app.scan();
      expect(app.scan()).toBe(scan);
      await Promise.resolve();
      publish.onmessage([{ tag: 'JAP', name: 'Japan' }, { tag: 'PRC', name: 'China' }]);
      app.countryQuery = 'jap';
      expect(app.visibleCountries.map(({ tag }) => tag)).toEqual(['JAP']);
      app.chooseCountry(app.visibleCountries[0]);
      expect(open).not.toHaveBeenCalled();
      finish(index);
      await expect(scan).resolves.toBe(true);
      expect(app.selectedCountry).toEqual(country);
      expect(app.gameText('[BHU.GetFlag]§Y[BHU.GetName]§!')).toEqual([
        { text: 'BHU', flag: true, icon: 'flag:BHU', color: undefined },
        { text: '不丹', color: '#ffd45a' },
      ]);
      expect(open).toHaveBeenCalledOnce();
      await expect(app.scan()).resolves.toBe(true);
      expect(backend).toHaveBeenCalledOnce();
      const rescan = app.scan(true);
      await Promise.resolve();
      finish(index);
      await expect(rescan).resolves.toBe(true);
      expect(backend).toHaveBeenCalledTimes(2);
    } finally {
      clearMocks();
    }
  });

  it('indexes minor countries only when requested and keeps the bookmark order', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.selectedMod = { name: 'Test mod', path: '/test-mod', workshop_id: '' };
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    const japan = { tag: 'JAP', name: 'Japan', chapter_count: 8, story_count: 8, major_order: 1 };
    const america = { tag: 'USA', name: 'America', chapter_count: 1, story_count: 1, major_order: 0 };
    const canada = { tag: 'CAN', name: 'Canada', chapter_count: 30, story_count: 30 };
    const secondary = vi.fn(() => ({ countries: [canada], cache_warning: null }));
    mockIPC((cmd) => cmd === 'index_secondary_countries' ? secondary() : {
      start_date: null, countries: [japan, america], chapters: [], timeline: [], diagnostics: [],
      has_secondary_countries: true,
    });
    try {
      await app.scan();
      expect(app.visibleCountries.map(({ tag }) => tag)).toEqual(['USA', 'JAP']);
      expect(secondary).not.toHaveBeenCalled();
      app.setShowAllCountries(true);
      await vi.waitFor(() => expect(app.visibleCountries.map(({ tag }) => tag)).toEqual(['USA', 'JAP', 'CAN']));
      app.setShowAllCountries(false);
      expect(app.visibleCountries.map(({ tag }) => tag)).toEqual(['USA', 'JAP']);
      app.setShowAllCountries(true);
      expect(secondary).toHaveBeenCalledOnce();
    } finally {
      clearMocks();
    }
  });

  it('applies the selected theme to the document and saves it', () => {
    const app = TestBed.createComponent(App).componentInstance;
    const persist = vi.spyOn(app, 'persist').mockImplementation(() => {});
    try {
      expect(app.saved.theme).toBe('auto');
      expect(app.theme).toBe('material');
      app.setTheme('tfr');
      expect(document.documentElement.dataset['readerTheme']).toBe('tfr');
      expect(app.saved.theme).toBe('tfr');
      app.setTheme('material');
      expect(document.documentElement.dataset['readerTheme']).toBe('material');
      expect(persist).toHaveBeenCalledTimes(2);
    } finally {
      delete document.documentElement.dataset['readerTheme'];
    }
  });

  it('follows the last played mod while allowing a fixed theme', () => {
    const app = TestBed.createComponent(App).componentInstance;
    vi.spyOn(app, 'openRoutePage').mockImplementation(() => {});
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    app.selectedMod = { name: '道阻且长', workshop_id: '3256452254', path: '/tno-child' };
    expect(app.theme).toBe('material');
    app.chooseCountry({ tag: 'USA', name: 'United States', chapter_count: 1, story_count: 1 });
    expect(app.theme).toBe('tno');
    expect(document.documentElement.dataset['readerTheme']).toBe('tno');
    app.setTheme('material');
    expect(app.theme).toBe('material');
    app.setTheme('auto');
    expect(app.theme).toBe('tno');
    delete document.documentElement.dataset['readerTheme'];
  });

  it('puts a TNO super event in its own panel and uses the local nuclear frame', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    fixture.detectChanges();
    await fixture.whenStable();
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    app.setTheme('tno');
    app.selectedMod = { name: 'The New Order', path: '/tno', workshop_id: '2438003901' };
    const image = 'data:image/gif;base64,R0lGODlhAQABAAAAACw=';
    const backend = vi.fn((command: string) => command === 'load_tno_super_event_skin'
      ? { frame: image, title: image, quote: image, button: image, nuclear_frame: image, nuclear_title: image }
      : image);
    mockIPC(backend);
    try {
      app.previewSuperEvent({ id: 'SE_NUCLEAR_WAR', title: 'Test event', quote: 'Quote line',
        dismiss: 'Continue', image_path: '/tno/gfx/art.png', audio: null });
      await vi.waitFor(() => {
        fixture.detectChanges();
        const panel = fixture.nativeElement.querySelector('.tno-super-event-panel') as HTMLElement;
        expect(panel.querySelector('.tno-super-event-picture')?.getAttribute('src')).toBe(image);
        expect(panel.querySelector('.tno-super-event-nuclear-frame')?.getAttribute('src')).toBe(image);
        expect(panel.querySelector('blockquote')?.textContent).toContain('Quote line');
      });
      expect(backend).toHaveBeenCalledWith('load_tno_super_event_skin', { modPath: '/tno' });
      (fixture.nativeElement.querySelector('#super-event-continue') as HTMLButtonElement).click();
      expect(app.superEvent).toBeNull();
    } finally {
      clearMocks();
      delete document.documentElement.dataset['readerTheme'];
    }
  });

  it('uses the Chinese TNO title and quote glyphs while preserving the original text', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    fixture.detectChanges();
    await fixture.whenStable();
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    app.setTheme('tno');
    app.saved.language = 'simp_chinese';
    app.selectedMod = { name: 'The New Order', path: '/tno', workshop_id: '2438003901' };
    const glyph = { image: 'data:image/png;base64,test', width: 1, height: 1, left: 0, bottom: 0, advance: 1 };
    const backend = vi.fn((command: string) => command === 'load_tno_super_event_glyphs'
      ? { title: { '战': glyph }, quote: { '和': glyph } }
      : null);
    mockIPC(backend);
    try {
      app.previewSuperEvent({ id: 'test', title: 'TNO战', quote: '§G和§!平',
        dismiss: '和', image_path: null, audio: null });
      await vi.waitFor(() => {
        fixture.detectChanges();
        const panel = fixture.nativeElement.querySelector('.tno-super-event-panel') as HTMLElement;
        const title = panel.querySelector('h1');
        const quote = panel.querySelector('blockquote');
        expect(title?.textContent).toBe('TNO战');
        expect(title?.querySelector('.tno-bitmap-character')?.textContent).toBe('战');
        expect(quote?.textContent).toBe('和平');
        expect(quote?.querySelector('.game-text-colored .tno-bitmap-character')?.textContent).toBe('和');
        expect(panel.querySelector('button .tno-bitmap-character')?.textContent).toBe('和');
      });
      expect(backend).toHaveBeenCalledWith('load_tno_super_event_glyphs', {
        modPath: '/tno', title: 'TNO战', quoteAndButton: '和平\n和',
      });
    } finally {
      clearMocks();
      delete document.documentElement.dataset['readerTheme'];
    }
  });

  it('renders flags in opening notes and previews search matches without advancing the route', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    fixture.detectChanges();
    await fixture.whenStable();
    app.saved.tag = 'CHI';
    app.saved.focus_id = '';
    app.selectedMod = { name: 'The Fire Rises', path: '/tfr-child', workshop_id: '3609728809' };
    app['openingNotes'] = {
      CHI: [{ text: '國家概況：\n[CHI.GetFlag]中華民國，與[PRC.GetFlag]中國。', images: [] }],
    };
    app.timeline = [{
      kind: 'event', id: 'lookup.1', title: 'Other story', description: '', country_tag: 'CHI',
      source: 'events.txt', timing: '', image_path: null, next_ids: [], linked_from: [],
      date: null, start_day: null, start_timing: null,
      options: [{ id: 'lookup.1.a', text: '考慮談判', next_ids: [] }],
    }];
    const flagImage = 'data:image/gif;base64,R0lGODlhAQABAAAAACw=';
    mockIPC((command) => command === 'load_inline_flag' ? flagImage : null);
    try {
      app.setPage('country-intro');
      fixture.detectChanges();
      await vi.waitFor(() => {
        fixture.detectChanges();
        const prose = fixture.nativeElement.querySelector('.country-intro .story-copy') as HTMLElement;
        expect([...prose.querySelectorAll('img.inline-game-flag')].map((image) => image.getAttribute('alt')))
          .toEqual(['CHI', 'PRC']);
        expect(prose.textContent).toContain('中華民國');
        expect(prose.textContent).not.toContain('GetFlag');
      });
      app.searchQuery = '國家概況';
      app.updateSearch();
      expect(app.searchResults.map(({ item }) => item.id)).toEqual(['opening:CHI:0']);
      app.searchQuery = '談判';
      app.updateSearch();
      expect(app.searchResults.map(({ item }) => item.id)).toEqual(['lookup.1']);
      app.previewSearchResult(app.searchResults[0]);
      expect(app.searchPreviewOptions[0].text).toBe('考慮談判');
      expect(app.saved.focus_id).toBe('');
      app.back();
      expect(app.page).toBe('search');
    } finally {
      clearMocks();
    }
  });

  it.each([['JAP', 'jap.1003'], ['PRC', 'china.2002']])('keeps %s epilogue behind the remaining focus chain', async (tag, endingId) => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = tag;
    app.saved.route_day = 100;
    const base = { title: '国策', description: '', icon: '', image_available: false, image_path: null,
      source: 'test.txt', focus_tree_id: 'campaign', initial_tree: true, duration_days: 7 };
    const root = { ...base, id: `${tag}_root`, event_calls: [[endingId, ''] as [string, string]] };
    const sibling = { ...base, id: `${tag}_sibling` };
    const followup = { ...base, id: `${tag}_followup`, prerequisite_groups: [[sibling.id]] };
    const locked = { ...base, id: `${tag}_locked`, prerequisite_groups: [[`${tag}_unselected`]] };
    const ending = { kind: 'event', id: endingId, country_tag: tag, title: '尾声', description: '',
      date: null, timing: '', source: 'test.txt', image_path: null, start_day: null, start_timing: null,
      linked_from: [], next_ids: [], options: [{ id: 'a', text: '继续', next_ids: [], credits: ['感谢游玩'] }] };
    app.chapters = [root, sibling, followup, locked];
    app.timeline = [ending];
    app['indexTimeline']();
    await app.read(root);
    expect(app.nextDueStory).toBeUndefined();
    expect(app.focusCandidates.map(({ id }) => id)).toEqual([sibling.id]);
    await app.read(sibling);
    expect(app.nextDueStory).toBeUndefined();
    expect(app.focusCandidates.map(({ id }) => id)).toEqual([followup.id]);
    await app.read(followup);
    expect(app.nextDueStory).toBe(ending);
    await app.readStory(ending);
    expect(app.currentCredits).toEqual(['感谢游玩']);
    expect(app.saved.read_focus_ids).not.toContain(locked.id);
  });

  it.each(['interrupt', 'queue'] as const)('switches story music in %s mode', async (mode) => {
    const app = TestBed.createComponent(App).componentInstance;
    const makeAudio = () => {
      const audio = document.createElement('audio');
      vi.spyOn(audio, 'play').mockResolvedValue();
      vi.spyOn(audio, 'pause').mockImplementation(() => {});
      vi.spyOn(audio, 'load').mockImplementation(() => {});
      return audio;
    };
    const first = makeAudio();
    const second = makeAudio();
    mockConvertFileSrc('windows');
    const players = [first, second];
    vi.stubGlobal('Audio', function () { return players.shift(); });
    vi.useFakeTimers();
    try {
      if (mode === 'queue') app.saved.music_transition = 'queue';
      expect(app.saved.music_transition).toBe(mode);
      const cue = (title: string) => app['enqueueSuperEventAudio']({ title, path: `${title}.ogg` });
      cue('First');
      cue('Second');
      if (mode === 'queue') {
        expect(first.pause).not.toHaveBeenCalled();
        expect(second.play).not.toHaveBeenCalled();
      } else {
        expect(first.pause).toHaveBeenCalledOnce();
        expect(second.play).toHaveBeenCalledOnce();
      }
      first.dispatchEvent(new Event('ended'));
      await Promise.resolve();
      expect(second.play).toHaveBeenCalledOnce();
      expect(app.bgmNotice).toBe('Second');
      expect(app.musicHistory[0]).toMatchObject({ title: 'Second', path: 'Second.ogg' });
      app.replayTrack(app.musicHistory[0]);
      expect(app.isPlayingTrack(app.musicHistory[0])).toBe(false);
    } finally {
      clearMocks();
      vi.useRealTimers();
      vi.unstubAllGlobals();
    }
  });

  it('shows the focus icon and prose with source details collapsed', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    app.ngOnInit = async () => {};
    app.page = 'reader';
    app.current = {
      id: 'JAP_test',
      title: '令和二年',
      description: '第一段。\n\n第二段。',
      icon: 'GFX_focus_test',
      image_available: true,
      image_path: null,
      source: 'common/national_focus/test.txt',
    };
    app.image = 'focus-icon.png';
    fixture.detectChanges();

    const reader = fixture.nativeElement.querySelector('.reader-page') as HTMLElement;
    expect(reader.querySelector('h1')?.textContent).toContain('令和二年');
    expect(reader.querySelector('.focus-reader-image img')?.getAttribute('src')).toBe('focus-icon.png');
    expect(reader.querySelector('.story-copy')?.textContent).toContain('第一段。\n\n第二段。');
    expect(reader.querySelector('details')?.open).toBe(false);
    expect(reader.querySelector('.reader-actions button')).toBeTruthy();
    expect(reader.querySelector('mat-chip')).toBeNull();
  });
  it('presents a focus decision as a checkbox and exposes the page control', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    fixture.detectChanges();
    await fixture.whenStable();
    app.saved.tag = 'JAP';
    app.saved.read_focus_ids = ['JAP_restrict_travel'];
    app.selectedMod = { name: 'The Fire Rises', path: '/tfr', workshop_id: '3350890356' };
    app.current = {
      id: 'JAP_restrict_travel', title: '限制旅行', description: '正文。',
      icon: '', image_available: false, image_path: null, source: 'focus.txt',
    };
    const countries = [['PRC', 'R', '中国'], ['SOV', 'G', '俄罗斯'], ['USA', 'C', '美国']];
    app.timeline = countries.map(([tag, color, name]) => ({
      id: `JAP_restrict_travel_${tag}`, kind: 'decision', country_tag: 'JAP',
      title: `限制[${tag}.GetFlag]§${color}${name}§!游客入境`,
      description: '[Missing translation: decision_desc]', date: null,
      timing: '', source: 'decisions.txt', image_path: null, start_day: null, start_timing: null,
      next_ids: [], linked_from: ['focus:JAP_restrict_travel (提示关联)'], options: [],
      decision_focuses: ['JAP_restrict_travel'],
    }));
    const flagImage = 'data:image/gif;base64,R0lGODlhAQABAAAAACw=';
    const backend = vi.fn((command: string) => command === 'load_inline_flag' ? flagImage : null);
    mockIPC(backend);
    try {
      app.setPage('reader');
      fixture.detectChanges();
      await fixture.whenStable();
      fixture.detectChanges();

      const reader = fixture.nativeElement.querySelector('.reader-page') as HTMLElement;
      const checkboxes = [...reader.querySelectorAll('mat-checkbox')];
      expect(checkboxes).toHaveLength(3);
      await vi.waitFor(() => {
        fixture.detectChanges();
        expect(reader.querySelectorAll('img.inline-game-flag')).toHaveLength(3);
      });
      for (const [tag, , name] of countries) {
        const checkbox = checkboxes.find((control) => control.textContent?.includes(name));
        expect(checkbox?.querySelector('img.inline-game-flag')?.getAttribute('src')).toBe(flagImage);
        expect(checkbox?.querySelector('img')?.getAttribute('alt')).toBe(tag);
        expect(checkbox?.querySelector('.game-text-colored')?.textContent).toBe(name);
        expect(backend).toHaveBeenCalledWith('load_inline_flag', expect.objectContaining({ tag }));
      }
      expect(reader.textContent).not.toMatch(/GetFlag|§|Missing translation/);
      expect(reader.querySelector('.reader-actions button')?.getAttribute('aria-label')).toBeTruthy();
      checkboxes[0].querySelector('input')?.click();
      expect(app.isDecisionChecked('JAP_restrict_travel_PRC')).toBe(true);
      checkboxes[0].querySelector('input')?.click();
      expect(app.isDecisionChecked('JAP_restrict_travel_PRC')).toBe(false);
    } finally {
      clearMocks();
    }
  });

  it('keeps equivalent response text visible while allowing quick continuation', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    app.current = {
      id: 'sample.1',
      kind: 'event',
      country_tag: 'JAP',
      title: 'A chapter',
      description: 'The story continues.',
      date: null,
      timing: '',
      source: 'events/sample.txt',
      image_path: null,
      start_day: null,
      start_timing: null,
      next_ids: ['generic.7'],
      linked_from: [],
      options: [
        { id: 'sample.1.a', text: 'Continue', next_ids: ['generic.7'], event_calls: [['generic.7', '']] },
        { id: 'sample.1.b', text: 'Close', next_ids: [] },
      ],
    };
    const [first, second] = app.currentOptions;
    expect(app.currentOptions.map(({ text }) => text)).toEqual(['Continue', 'Close']);
    expect(app.hasStoryBranches).toBe(false);
    const choose = vi.spyOn(app, 'chooseStoryOption').mockResolvedValue();
    await app.continueRoute();
    expect(choose).toHaveBeenCalledWith(first, false);
    choose.mockClear();

    second.mission_time_changes = [['mission.sample', -5]];
    expect(app.hasStoryBranches).toBe(true);
    await app.continueRoute();
    expect(choose).not.toHaveBeenCalled();
  });

  it('shows an authored single response and applies it only once when clicked', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    app.ngOnInit = async () => {};
    vi.spyOn(app, 'persist').mockImplementation(() => {});
    vi.spyOn(app, 'openRoutePage').mockImplementation(() => {});
    fixture.detectChanges();
    app.saved.tag = 'JAP';
    app.saved.read_story_ids = [];
    app.saved.mission_started = { 'mission.sample': 0 };
    app.current = {
      id: 'sample.2', kind: 'event', country_tag: 'JAP', title: 'A letter',
      description: 'The story continues.', date: null, timing: '', source: 'events/sample.txt',
      image_path: null, start_day: null, start_timing: null, next_ids: [], linked_from: [],
      options: [{ id: 'sample.2.a', text: 'File the letter', next_ids: [], mission_time_changes: [['mission.sample', -5]] }],
    };
    app.setPage('reader');
    fixture.detectChanges();

    const response = fixture.nativeElement.querySelector('.reader-article .story-option') as HTMLButtonElement;
    const shortcut = fixture.nativeElement.querySelector('.reader-actions button') as HTMLButtonElement;
    expect(response.textContent?.trim()).toBe('File the letter');
    expect(shortcut.getAttribute('aria-label')).toBe('File the letter');
    expect(app.saved.mission_deadline_adjustments?.['mission.sample']).toBeUndefined();
    response.click();
    response.click();
    expect(app.saved.story_choices?.['event:sample.2']).toBe('sample.2.a');
    expect(app.saved.mission_deadline_adjustments?.['mission.sample']).toBe(-5);
  });

  it('dates a verified random delay from its script and labels the estimate', () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'ITA';
    const event = {
      id: 'vatican.test',
      kind: 'event',
      country_tag: 'ITA',
      title: 'Test',
      description: 'Test',
      date: null,
      timing: '',
      source: 'events/test.txt',
      image_path: null,
      start_day: null,
      start_timing: null,
      next_ids: [],
      linked_from: ['startup:ITA (days 30, random_days 10)'],
      verified_startup: [{ recipient: 'ITA', target: 'vatican.test', delay: 'days 30, random_days 10' }],
      options: [],
    };
    expect(app.storyDay(event)).toBe(35);
    expect(app.isProjectedStory(event)).toBe(true);
  });

  it('keeps the drawer closed from the reader through an edge-to-edge focus choice', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    app.ngOnInit = async () => {};
    app.saved.tag = 'JAP';
    app.chapters = Array.from({ length: 5 }, (_, index) => ({
      id: `JAP_test_${index}`, title: `国策 ${index}`, description: '国策描述',
      icon: '', image_available: false, image_path: null, source: 'focus.txt',
      initial_tree: true, focus_tree_id: 'test', x: index, y: 0,
    }));
    app.openRoutePage();
    fixture.detectChanges();

    const chooser = fixture.nativeElement.querySelector('.focus-picker') as HTMLElement;
    expect(app.immersiveReading).toBe(true);
    expect(chooser.querySelectorAll('.focus-choice-tile')).toHaveLength(5);
    expect([...chooser.querySelectorAll('.focus-choice-row')].map((row) =>
      row.querySelectorAll('button').length,
    )).toEqual([3, 2]);
    expect(fixture.nativeElement.querySelector('mat-toolbar')).toBeNull();
    expect(chooser.querySelector('mat-card, details')).toBeNull();
    (chooser.querySelector('.focus-choice-tile') as HTMLButtonElement).click();
    fixture.detectChanges();
    expect(app.immersiveReading).toBe(true);
    expect(fixture.nativeElement.querySelector('.reader-page h1')?.textContent).toBe('国策 0');
  });

  it('names each mutually exclusive alternative and leaves independent focuses unlabeled', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    app.ngOnInit = async () => {};
    app.saved.tag = 'JAP';
    const focus = {
      description: '国策描述', icon: '', image_available: false, image_path: null,
      source: 'focus.txt', initial_tree: true, focus_tree_id: 'test',
    };
    app.chapters = [
      { ...focus, id: 'JAP_a', title: '路线甲', mutually_exclusive_ids: ['JAP_b'], x: 0, y: 0 },
      { ...focus, id: 'JAP_c', title: '独立国策', x: 1, y: 0 },
      { ...focus, id: 'JAP_b', title: '路线乙', x: 2, y: 1 },
    ];
    app.openRoutePage();
    fixture.detectChanges();
    const tiles = [...fixture.nativeElement.querySelectorAll('.focus-choice-tile')] as HTMLElement[];
    expect(tiles.map((tile) => tile.querySelector('.focus-choice-branch span')?.textContent?.trim()))
      .toEqual(['与「路线乙」互斥', '与「路线甲」互斥', undefined]);
  });

  it('automatically schedules a decision from its completed focus only once', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    const base = {
      title: '到访武汉', description: '', country_tag: 'PRC', date: null, timing: '',
      source: 'decisions.txt', image_path: null, start_day: null, start_timing: null,
      linked_from: [], next_ids: [], options: [],
    };
    const event = { ...base, kind: 'event', id: 'china.74' };
    const decision = {
      ...base, kind: 'decision', id: 'PRC_visit_wuhan', decision_focuses: ['PRC_lockdown_wuhan'],
      decision_condition: { op: 'check' as const, args: { flag: 'focus:PRC_lockdown_wuhan', present: true } },
      decision_event_calls: [['china.74', 'days 5'] as [string, string]],
    };
    app.saved.tag = 'PRC';
    app.saved.route_day = 100;
    app.timeline = [decision, event];
    app['indexTimeline']();
    app['advanceAutomaticDecisions']();
    expect(app.saved.decision_started?.[decision.id]).toBeUndefined();
    app.saved.read_focus_ids = ['PRC_lockdown_wuhan'];
    app.saved.focus_completion_days = { PRC_lockdown_wuhan: 90 };
    app['advanceAutomaticDecisions']();
    app['advanceAutomaticDecisions']();
    expect(app.storyDay(event)).toBe(95);
    expect(app.saved.read_story_ids.filter((key) => key === 'decision:PRC_visit_wuhan')).toHaveLength(1);
  });

  it('waits for a weekly hook condition and schedules the event only for its recipient', () => {
    const app = TestBed.createComponent(App).componentInstance;
    const event = {
      kind: 'event', id: 'sample.1', country_tag: 'GER', title: 'Story', description: '',
      date: null, timing: '', source: 'events/sample.txt', image_path: null,
      start_day: null, start_timing: null, linked_from: [], next_ids: [], options: [],
      periodic_triggers: [{
        owner: 'GER', recipient: 'GER', interval: 7, delay: 'days 2',
        condition: { op: 'check' as const, args: { flag: 'sample_unlocked', present: true } },
      }],
    };
    app.saved.tag = 'GER';
    app.saved.route_day = 100;
    app.timeline = [event];
    app['indexTimeline']();
    app['schedulePeriodicStories']();
    expect(app.saved.option_due_days?.['event:sample.1']).toBeUndefined();
    app.saved.country_flag_changes = { sample_unlocked: true };
    app['schedulePeriodicStories']();
    expect(app.storyDay(event)).toBe(107);
    expect(app.isProjectedStory(event)).toBe(true);
  });

  it('waits until the projected foreign war ends before asking for its result', () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'GER';
    app['startDate'] = '2020.1.1';
    app['greatAsianWarNews'] = ['news.415', ''];
    const base = { country_tag: 'PRC', title: '', description: '', date: null, timing: '', source: 'test.txt',
      image_path: null, start_day: null, start_timing: null, linked_from: [], next_ids: [], options: [] };
    const win = { ...base, id: 'news.413', kind: 'news', broadcast: true, startup_trigger: '' };
    const loss = { ...base, id: 'news.414', kind: 'news', broadcast: true, startup_trigger: '' };
    const blank = { calls: [] as [string, string][], foreign_events: [], flags: [], trees: [], completions: [] };
    const mission = { ...base, id: 'PRC_GAW_war_exhaustion', kind: 'mission', mission: {
      days: 200, complete: { ...blank, calls: [['news.413', 'days 1']] as [string, string][] },
      cancel: { ...blank, calls: [['news.414', 'days 1']] as [string, string][] }, timeout: blank,
    } };
    app.timeline = [mission, win, loss];
    app['indexTimeline']();
    const deadline = app['calendarDay']('2028.1.1')! + 200;
    app.saved.route_day = deadline - 1;
    void app.readStory(mission);
    expect(app.currentMissionChoices).toEqual([]);
    expect(app.saved.option_due_days?.['news:news.414']).toBeUndefined();
    app.saved.route_day = deadline;
    expect(app.currentMissionChoices.map(({ id }) => id)).toEqual(['complete', 'cancel']);
    vi.spyOn(app, 'continueRoute').mockResolvedValue();
    app.resolveMission('complete');
    expect(app.storyDay(win)).toBe(deadline + 1);
    expect(app.isProjectedStory(win)).toBe(true);
    expect(app.saved.option_due_days?.['news:news.414']).toBeUndefined();
    app.undoRoute();
    expect(app.saved.option_due_days?.['news:news.413']).toBeUndefined();
    expect(app.currentMissionChoices).toHaveLength(2);
  });

  it('advances to a foreign focus completion and applies its domestic effects once', () => {
    const app = TestBed.createComponent(App).componentInstance;
    const base = { title: 'Test', description: '', icon: '', image_available: false,
      image_path: null, source: 'focus.txt', initial_tree: true, background_availability: [] };
    app.saved.tag = 'GER';
    app.saved.route_day = 100;
    app.chapters = [
      { ...base, id: 'SOV_war', focus_tree_id: 'sov', duration_days: 120,
        focus_completions: [{ tag: 'GER', id: 'GER_defend' }] },
      { ...base, id: 'GER_defend', focus_tree_id: 'ger', never_manually_available: true,
        event_calls: [['example.1', 'days 4']] },
      { ...base, id: 'GER_followup', focus_tree_id: 'ger', prerequisite_groups: [['GER_defend']] },
    ];
    const event = { ...base, id: 'example.1', kind: 'event', country_tag: 'GER', date: null,
      timing: '', start_day: null, start_timing: null, next_ids: [], linked_from: [], options: [] };
    app.timeline = [event];
    app['indexTimeline']();
    expect(app.nextDevelopmentDay).toBe(120);
    expect(app.saved.read_focus_ids).not.toContain('GER_defend');
    app.advanceToNextStory();
    expect(app.saved.focus_completion_days['GER_defend']).toBe(120);
    expect(app.storyDay(event)).toBe(124);
    expect(app.isProjectedStory(event)).toBe(true);
    expect(app.focusCandidates.map(({ id }) => id)).toContain('GER_followup');
    app['advanceAutomaticDecisions']();
    expect(app.saved.read_focus_ids.filter((id) => id === 'GER_defend')).toHaveLength(1);
  });

  it('asks at the script date and keeps the other outcome out of the calendar after a choice', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'BEL';
    app.saved.route_day = 100;
    app['startDate'] = '2020.1.1';
    const event = { id: 'sample.1', kind: 'event', country_tag: 'FRA', title: '佛兰德斯', description: '',
      date: null, timing: '', source: 'test.txt', image_path: null, start_day: null, start_timing: null,
      next_ids: [], linked_from: [], options: [],
      manual_trigger: { owner: 'BEL', earliest: '2020.6.1', earliest_inclusive: false, latest: null,
        latest_inclusive: false, focuses: [], condition: { op: 'unknown' as const } } };
    app.timeline = [event];
    app['indexTimeline']();
    app['scheduleAnchoredConfirmations']();
    const day = app['calendarDay']('2020.6.2')!;
    expect(app.nextDevelopmentDay).toBe(day);
    expect(app.nextDueStory).toBeUndefined();
    app.advanceToNextStory();
    expect(app.currentConfirmation?.choices.map(({ label }) => label)).toEqual(['发生「佛兰德斯」', '条件未达成，继续阅读']);
    app.resolveConfirmation(1);
    expect(app.saved.option_due_days?.['event:sample.1']).toBeUndefined();
    app.undoRoute();
    expect(app.currentConfirmation?.choices).toHaveLength(2);
    app.resolveConfirmation(0);
    expect(app.storyDay(event)).toBe(day);
    expect(app.saved.read_story_ids).toContain('event:sample.1');
  });

  it('executes only the chosen unresolved arm of a timed mission', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'GER';
    app.saved.route_day = 150;
    const base = { title: '', description: '', country_tag: 'GER', date: null, timing: '', source: 'test.txt',
      image_path: null, start_day: null, start_timing: null, linked_from: [], next_ids: [], options: [] };
    const first = { ...base, id: 'sample.1', kind: 'news', title: '甲方胜利', broadcast: true };
    const other = { ...base, id: 'sample.2', kind: 'news', title: '乙方胜利', broadcast: true };
    app.timeline = [first, other];
    app['indexTimeline']();
    const arm = { trees: [], completions: [], owner: 'SOV' };
    app['applyConditionalRoutes']([[{ ...arm, condition: { op: 'unknown' }, calls: [['sample.1', 'days 2']] },
      { ...arm, condition: { op: 'all', args: [] }, calls: [['sample.2', 'days 2']] }]],
      150, 'SOV', true, { ...base, id: 'sample_mission', kind: 'mission' });
    await app.readStory(app.nextDueStory!);
    expect(app.currentConfirmation?.choices.map(({ label }) => label)).toEqual(['甲方胜利', '乙方胜利']);
    app.resolveConfirmation(1);
    expect(app.saved.option_due_days?.['news:sample.1']).toBeUndefined();
    expect(app.saved.option_due_days?.['news:sample.2']).toBe(152);
    expect(app.isProjectedStory(other)).toBe(true);
  });

  it('asks once for the linked environmental outcomes and does not queue the wildlife events again', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'PRC';
    app.saved.route_day = 150;
    const base = { title: '维护两座大山理论', description: '', country_tag: 'PRC', date: null, timing: '',
      source: 'test.txt', image_path: null, start_day: null, start_timing: null, linked_from: [], next_ids: [], options: [] };
    const monkey = { ...base, id: 'china.223', kind: 'event', title: '金丝猴濒临灭绝' };
    const alligator = { ...base, id: 'china.225', kind: 'event', title: '扬子鳄被宣布灭绝' };
    app.timeline = [monkey, alligator];
    app['indexTimeline']();
    const routes = [[{ condition: { op: 'check' as const, args: { flag: 'reader:PRC_environment_30', present: true } },
      trees: [], completions: [], calls: [['china.223', 'days 6']] as [string, string][] }],
    [{ condition: { op: 'check' as const, args: { flag: 'reader:PRC_environment_20', present: true } },
      trees: [], completions: [], calls: [['china.225', 'days 45']] as [string, string][] }]];
    const focus = { ...base, id: 'PRC_preserve_two_mountain_theory', kind: 'event' };
    app['applyConditionalRoutes'](routes, 150, 'PRC', false, focus);
    app['applyConditionalRoutes'](routes, 155, 'PRC', false, focus);
    expect(Object.keys(app.saved.confirmations ?? {})).toEqual(['confirmation:PRC_environment_outcome']);
    await app.readStory(app.nextDueStory!);
    app.resolveConfirmation(2);
    expect(app.storyDay(monkey)).toBe(156);
    expect(app.storyDay(alligator)).toBe(195);
    app['applyConditionalRoutes'](routes, 170, 'PRC', false, focus);
    expect(app.storyDay(monkey)).toBe(156);
    expect(Object.keys(app.saved.confirmations ?? {})).toHaveLength(1);
  });

  it('places territorial developments inside the war and allows one result per hook', async () => {
    const app = TestBed.createComponent(App).componentInstance;
    app.saved.tag = 'FRA';
    app.saved.route_day = 100;
    const base = { title: 'War', description: '', country_tag: 'FRA', date: null, timing: '', source: 'test.txt',
      image_path: null, start_day: null, start_timing: null, linked_from: [], next_ids: [], options: [] };
    const news = { ...base, id: 'news.237', kind: 'news', broadcast: true };
    const mission = { ...base, id: 'SOV_nato_war_deadline', kind: 'mission', mission: {
      days: 80, complete: { calls: [], foreign_events: [], flags: [], trees: [], completions: [] },
      timeout: { calls: [], foreign_events: [], flags: [], trees: [], completions: [] },
      cancel: { calls: [], foreign_events: [], flags: [], trees: [], completions: [] },
    } };
    const lost = { ...base, id: 'france.1', kind: 'event', title: '失守' };
    const reclaimed = { ...base, id: 'france.2', kind: 'event', title: '夺回' };
    app.timeline = [news, mission, lost, reclaimed];
    app['indexTimeline']();
    app.saved.mission_started = { SOV_nato_war_deadline: 100 };
    app['wartimeHooks'] = [{ id: 'territory', war: 'SOV_nato_war_begins_global', kind: 'on_state_control_changed', source: 'test.txt',
      arms: [lost, reclaimed].map(({ id }) => ({ condition: { op: 'unknown' }, flags: [], calls: [{ recipient: 'FRA', target: id, delay: 'days 2' }] })) }];
    app['recordWarStart'](news, 100);
    const entry = Object.values(app.saved.confirmations ?? {})[0];
    expect(entry.day).toBe(140);
    expect(app.nextDueStory).toBeUndefined();
    app.saved.route_day = 140;
    await app.readStory(entry.item);
    app.setWartimeChoice(0, true);
    app.setWartimeChoice(1, true);
    app.resolveWartimeChoices();
    expect(app.saved.option_due_days?.['event:france.1']).toBeUndefined();
    expect(app.storyDay(reclaimed)).toBe(142);
    app.undoRoute();
    expect(app.saved.option_due_days?.['event:france.2']).toBeUndefined();
  });
});
